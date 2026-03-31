use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}

pub fn render(app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 8, frame.area());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" bankai ");

    let mut lines = vec![
        Line::from(""),
        Line::from("AniList Terminal Client"),
        Line::from(""),
        Line::from("Press Enter to login with AniList"),
        Line::from("Press q to quit"),
    ];

    if let Some(ref msg) = app.status_message {
        lines.push(Line::from(""));
        lines.push(Line::styled(msg.clone(), Style::default().fg(Color::Yellow)));
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}
