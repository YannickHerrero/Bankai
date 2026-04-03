use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::api::MediaListStatus;
use crate::app::{App, SearchFocus, SearchPopup};
use crate::ui::centered_rect;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    render_search_bar(app, frame, chunks[0]);

    if app.search.results.is_empty() {
        render_empty_body(app, frame, chunks[1]);
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        render_results_list(app, frame, body[0]);
        render_detail_panel(app, frame, body[1]);
    }

    if let Some(ref popup) = app.search.popup {
        match popup {
            SearchPopup::StatusPicker { selected } => {
                render_status_picker(*selected, frame);
            }
            SearchPopup::RemoveConfirm { confirm_selected } => {
                render_remove_confirm(app, *confirm_selected, frame);
            }
        }
    }
}

fn render_search_bar(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.search.focus == SearchFocus::Input;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let border_type = if focused {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(border_type)
        .title(" Search ");

    let anime_style = if app.search.media_type == crate::app::SearchMediaType::Anime {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let manga_style = if app.search.media_type == crate::app::SearchMediaType::Manga {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut spans = vec![
        Span::styled(" [", Style::default().fg(Color::DarkGray)),
        Span::styled("Anime", anime_style),
        Span::styled("] [", Style::default().fg(Color::DarkGray)),
        Span::styled("Manga", manga_style),
        Span::styled("]  ", Style::default().fg(Color::DarkGray)),
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::raw(&app.search.query),
    ];

    if focused {
        spans.push(Span::styled(
            "█",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
    }

    if app.search.searching {
        spans.push(Span::styled(
            "  Searching...",
            Style::default().fg(Color::Yellow),
        ));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(block);
    frame.render_widget(paragraph, area);
}

fn render_empty_body(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Results ");

    let msg = if app.search.searching {
        "Searching..."
    } else if app.search.query.is_empty() {
        "Type a query and press Enter to search"
    } else {
        "No results found"
    };

    let text = Paragraph::new(Line::from(Span::styled(
        msg,
        Style::default().fg(Color::DarkGray),
    )))
    .block(block)
    .alignment(ratatui::layout::Alignment::Center);

    let popup = centered_rect(area.width.min(50), 3, area);
    frame.render_widget(text, popup);
}

fn render_results_list(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.search.focus == SearchFocus::Results;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let border_type = if focused {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(border_type)
        .title(" Results ");

    let items: Vec<ListItem> = app
        .search
        .results
        .iter()
        .map(|media| {
            let mut spans = Vec::new();

            // On-list indicator
            if media.media_list_entry.is_some() {
                spans.push(Span::styled("● ", Style::default().fg(Color::Cyan)));
            } else {
                spans.push(Span::raw("  "));
            }

            // Title
            spans.push(Span::styled(
                &media.title.romaji,
                Style::default().fg(Color::White),
            ));

            // Format
            if let Some(ref fmt) = media.format {
                spans.push(Span::styled(
                    format!("  {fmt}"),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Year
            if let Some(year) = media.season_year {
                spans.push(Span::styled(
                    format!("  {year}"),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Score
            if let Some(score) = media.average_score {
                spans.push(Span::styled(
                    format!("  ★ {score}"),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default();
    state.select(Some(app.search.result_scroll));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_detail_panel(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.search.focus == SearchFocus::Detail;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let border_type = if focused {
        BorderType::Thick
    } else {
        BorderType::Plain
    };

    let media = match app.search.selected_media() {
        Some(m) => m,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .border_type(border_type)
                .title(" Detail ");
            frame.render_widget(block, area);
            return;
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(border_type)
        .title(format!(" {} ", media.title.romaji));

    let mut lines: Vec<Line> = Vec::new();

    // Title
    lines.push(Line::from(Span::styled(
        &media.title.romaji,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));

    // English title
    if let Some(ref english) = media.title.english {
        if english != &media.title.romaji {
            lines.push(Line::from(Span::styled(
                english,
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));

    // Info rows
    let mut info_spans: Vec<Span> = Vec::new();
    if let Some(ref fmt) = media.format {
        info_spans.push(Span::styled(fmt, Style::default().fg(Color::White)));
    }
    if let Some(eps) = media.episodes {
        if !info_spans.is_empty() {
            info_spans.push(Span::styled(" · ", Style::default().fg(Color::DarkGray)));
        }
        info_spans.push(Span::styled(
            format!("{eps} episodes"),
            Style::default().fg(Color::White),
        ));
    }
    if let Some(ch) = media.chapters {
        if !info_spans.is_empty() {
            info_spans.push(Span::styled(" · ", Style::default().fg(Color::DarkGray)));
        }
        info_spans.push(Span::styled(
            format!("{ch} chapters"),
            Style::default().fg(Color::White),
        ));
    }
    if let Some(score) = media.average_score {
        if !info_spans.is_empty() {
            info_spans.push(Span::styled(" · ", Style::default().fg(Color::DarkGray)));
        }
        info_spans.push(Span::styled(
            format!("★ {score}"),
            Style::default().fg(Color::Yellow),
        ));
    }
    if !info_spans.is_empty() {
        lines.push(Line::from(info_spans));
    }

    // Status and year
    let mut meta_spans: Vec<Span> = Vec::new();
    if let Some(ref status) = media.status {
        meta_spans.push(Span::styled(
            format!("Status: {status}"),
            Style::default().fg(Color::DarkGray),
        ));
    }
    if let Some(year) = media.season_year {
        if !meta_spans.is_empty() {
            meta_spans.push(Span::styled("  ·  ", Style::default().fg(Color::DarkGray)));
        }
        meta_spans.push(Span::styled(
            format!("Year: {year}"),
            Style::default().fg(Color::DarkGray),
        ));
    }
    if !meta_spans.is_empty() {
        lines.push(Line::from(meta_spans));
    }

    // Genres
    if let Some(ref genres) = media.genres {
        if !genres.is_empty() {
            lines.push(Line::from(Span::styled(
                genres.join(", "),
                Style::default().fg(Color::Cyan),
            )));
        }
    }

    lines.push(Line::from(""));

    // Description
    if let Some(ref desc) = media.description {
        let cleaned = strip_html(desc);
        for line in cleaned.lines() {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::White),
            )));
        }
    }

    // User list section
    if let Some(ref entry) = media.media_list_entry {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "── Your List ──────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(vec![
            Span::styled("  Status:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(&entry.status, Style::default().fg(Color::Cyan)),
        ]));
        let progress_text = if let Some(eps) = media.episodes {
            format!("{}/{}", entry.progress, eps)
        } else if let Some(ch) = media.chapters {
            format!("{}/{}", entry.progress, ch)
        } else {
            format!("{}", entry.progress)
        };
        lines.push(Line::from(vec![
            Span::styled("  Progress: ", Style::default().fg(Color::DarkGray)),
            Span::styled(progress_text, Style::default().fg(Color::White)),
        ]));
        if entry.score > 0.0 {
            lines.push(Line::from(vec![
                Span::styled("  Score:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("★ {}", entry.score),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
        }
    }

    // Footer hints
    lines.push(Line::from(""));
    if focused {
        let hint = if media.media_list_entry.is_some() {
            "[a] Remove from list  [Esc] Back"
        } else {
            "[a] Add to list  [Esc] Back"
        };
        lines.push(Line::from(Span::styled(
            hint,
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "[Enter] Open detail",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((app.search.detail_scroll as u16, 0));

    frame.render_widget(paragraph, area);
}

fn render_status_picker(selected: usize, frame: &mut Frame) {
    let statuses = MediaListStatus::ALL;
    let height = statuses.len() as u16 + 4;
    let popup_area = centered_rect(30, height, frame.area());

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Add to List ");

    let items: Vec<ListItem> = statuses
        .iter()
        .map(|s| {
            ListItem::new(Line::from(Span::styled(
                format!(" {} ", s.label()),
                Style::default().fg(Color::White),
            )))
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, popup_area, &mut state);
}

fn render_remove_confirm(app: &App, confirm_selected: bool, frame: &mut Frame) {
    let popup_area = centered_rect(45, 6, frame.area());

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Remove from List ");

    let title = app
        .search
        .selected_media()
        .map(|m| m.title.romaji.as_str())
        .unwrap_or("this media");

    let yes_style = if confirm_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let no_style = if !confirm_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let lines = vec![
        Line::from(Span::styled(
            format!(" Remove \"{}\"?", truncate_str(title, 35)),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("       "),
            Span::styled(" Yes ", yes_style),
            Span::raw("     "),
            Span::styled(" No ", no_style),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup_area);
}

fn strip_html(html: &str) -> String {
    let s = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<i>", "")
        .replace("</i>", "")
        .replace("<b>", "")
        .replace("</b>", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&apos;", "'");

    // Strip remaining HTML tags
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
