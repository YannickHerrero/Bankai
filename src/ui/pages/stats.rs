use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, StatsSection};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_overview(app, frame, top[0]);
    render_score_distribution(app, frame, top[1]);
    render_top_genres(app, frame, bottom[0]);
    render_formats(app, frame, bottom[1]);
}

fn section_block<'a>(app: &'a App, section: StatsSection, title: &'a str) -> Block<'a> {
    let focused = app.stats_section == section;
    if focused {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title)
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(title)
    }
}

fn format_number(n: i64) -> String {
    if n < 0 {
        return format!("-{}", format_number(-n));
    }
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn status_label(status: &str) -> &str {
    match status {
        "CURRENT" => "Watching",
        "PLANNING" => "Planning",
        "COMPLETED" => "Completed",
        "PAUSED" => "Paused",
        "DROPPED" => "Dropped",
        other => other,
    }
}

fn render_overview(app: &App, frame: &mut Frame, area: Rect) {
    let block = section_block(app, StatsSection::Overview, " Overview ");

    let stats = match &app.stats_data {
        Some(s) => &s.anime,
        None => {
            let paragraph = Paragraph::new("Loading statistics...").block(block);
            frame.render_widget(paragraph, area);
            return;
        }
    };

    let days = stats.minutes_watched as f64 / 1440.0;

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Total Anime: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_number(stats.count as i64),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Episodes: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format_number(stats.episodes_watched as i64),
                Style::default().fg(Color::White),
            ),
            Span::styled("  ·  Days: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{days:.1}"),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Mean Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}", stats.mean_score),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("  (±{:.1})", stats.standard_deviation),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(Span::styled(
            "  ────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    // Status breakdown in two-column layout
    let statuses: Vec<(&str, i32)> = stats
        .statuses
        .iter()
        .map(|s| (status_label(&s.status), s.count))
        .collect();

    // Render statuses in pairs for compact layout
    let mut i = 0;
    while i < statuses.len() {
        let mut spans = vec![Span::raw("  ")];
        spans.push(Span::styled(
            format!("{}: ", statuses[i].0),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::styled(
            format!("{:<6}", statuses[i].1),
            Style::default().fg(Color::White),
        ));
        if i + 1 < statuses.len() {
            spans.push(Span::styled(
                format!("  {}: ", statuses[i + 1].0),
                Style::default().fg(Color::DarkGray),
            ));
            spans.push(Span::styled(
                format!("{}", statuses[i + 1].1),
                Style::default().fg(Color::White),
            ));
        }
        lines.push(Line::from(spans));
        i += 2;
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((app.stats_overview_scroll as u16, 0));
    frame.render_widget(paragraph, area);
}

fn render_score_distribution(app: &App, frame: &mut Frame, area: Rect) {
    let block = section_block(app, StatsSection::ScoreDistribution, " Score Distribution ");

    let stats = match &app.stats_data {
        Some(s) => &s.anime,
        None => {
            let paragraph = Paragraph::new("Loading...").block(block);
            frame.render_widget(paragraph, area);
            return;
        }
    };

    if stats.scores.is_empty() {
        let paragraph = Paragraph::new("  No score data").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let bars: Vec<Bar> = stats
        .scores
        .iter()
        .map(|s| {
            Bar::default()
                .value(s.count as u64)
                .label(Line::from(s.score.to_string()))
                .style(Style::default().fg(Color::Cyan))
                .value_style(Style::default().fg(Color::Yellow))
        })
        .collect();

    let chart = BarChart::default()
        .block(block)
        .data(BarGroup::default().bars(&bars))
        .bar_width(3)
        .bar_gap(1);

    frame.render_widget(chart, area);
}

fn render_top_genres(app: &App, frame: &mut Frame, area: Rect) {
    let block = section_block(app, StatsSection::TopGenres, " Top Genres ");

    let stats = match &app.stats_data {
        Some(s) => &s.anime,
        None => {
            let paragraph = Paragraph::new("Loading...").block(block);
            frame.render_widget(paragraph, area);
            return;
        }
    };

    if stats.genres.is_empty() {
        let paragraph = Paragraph::new("  No genre data").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let max_label_len = stats
        .genres
        .iter()
        .map(|g| g.genre.len())
        .max()
        .unwrap_or(0);

    let bars: Vec<Bar> = stats
        .genres
        .iter()
        .map(|g| {
            Bar::default()
                .value(g.count as u64)
                .label(Line::from(format!("{:>width$}", g.genre, width = max_label_len)))
                .style(Style::default().fg(Color::Cyan))
                .value_style(Style::default().fg(Color::White))
        })
        .collect();

    let chart = BarChart::default()
        .block(block)
        .data(BarGroup::default().bars(&bars))
        .direction(Direction::Horizontal)
        .bar_width(1)
        .bar_gap(0);

    frame.render_widget(chart, area);
}

fn render_formats(app: &App, frame: &mut Frame, area: Rect) {
    let block = section_block(app, StatsSection::Formats, " Formats ");

    let stats = match &app.stats_data {
        Some(s) => &s.anime,
        None => {
            let paragraph = Paragraph::new("Loading...").block(block);
            frame.render_widget(paragraph, area);
            return;
        }
    };

    if stats.formats.is_empty() {
        let paragraph = Paragraph::new("  No format data").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let max_label_len = stats
        .formats
        .iter()
        .map(|f| f.format.len())
        .max()
        .unwrap_or(0);

    let bars: Vec<Bar> = stats
        .formats
        .iter()
        .map(|f| {
            Bar::default()
                .value(f.count as u64)
                .label(Line::from(format!("{:>width$}", f.format, width = max_label_len)))
                .style(Style::default().fg(Color::Cyan))
                .value_style(Style::default().fg(Color::White))
        })
        .collect();

    let chart = BarChart::default()
        .block(block)
        .data(BarGroup::default().bars(&bars))
        .direction(Direction::Horizontal)
        .bar_width(1)
        .bar_gap(0);

    frame.render_widget(chart, area);
}
