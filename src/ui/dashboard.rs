use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
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

    let text = if app.watching_list.is_empty() {
        "No anime in watching list".to_string()
    } else {
        format!("{} anime", app.watching_list.len())
    };

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
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
