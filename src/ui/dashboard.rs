use ratatui::Frame;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" bankai - Dashboard ");

    let text = if let Some(ref username) = app.username {
        format!("Welcome, {username}!")
    } else if app.loading {
        "Loading...".to_string()
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, frame.area());
}
