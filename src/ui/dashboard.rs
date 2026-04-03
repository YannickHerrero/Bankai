use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, DashboardSection};

pub fn render(app: &App, frame: &mut Frame) {
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(frame.area());

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);

    render_watching(app, frame, outer[0]);
    render_calendar(app, frame, right[0]);
    render_updates(app, frame, right[1]);
}

fn section_style(app: &App, section: DashboardSection) -> Style {
    if app.dashboard_section == section {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn render_watching(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(section_style(app, DashboardSection::Watching))
        .title(" Currently Watching ");

    if app.watching_list.is_empty() {
        let paragraph = Paragraph::new("No anime in watching list").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .watching_list
        .iter()
        .map(|entry| {
            let title = &entry.media.title.romaji;
            let total = entry
                .media
                .episodes
                .map(|e| e.to_string())
                .unwrap_or_else(|| "?".to_string());
            let score_str = if entry.score > 0.0 {
                format!("  ★ {}", entry.score as u8)
            } else {
                String::new()
            };
            let line = Line::from(vec![
                Span::styled(
                    format!(" {title}"),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("  ({}/{})", entry.progress, total),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(score_str, Style::default().fg(Color::Yellow)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(app.watching_scroll));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_calendar(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(section_style(app, DashboardSection::Calendar))
        .title(" Weekly Calendar ");

    let paragraph = Paragraph::new("No shows airing this week").block(block);
    frame.render_widget(paragraph, area);
}

fn render_updates(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(section_style(app, DashboardSection::Updates))
        .title(" Last Updates ");

    let text = if app.recent_activity.is_empty() {
        "No recent activity".to_string()
    } else {
        format!("{} activities", app.recent_activity.len())
    };

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}
