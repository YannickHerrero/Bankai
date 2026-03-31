mod login;

use ratatui::Frame;
use ratatui::widgets::Paragraph;

use crate::app::{App, AppScreen};

pub fn render(app: &App, frame: &mut Frame) {
    match app.screen {
        AppScreen::Login => login::render(app, frame),
        AppScreen::Dashboard => {
            frame.render_widget(Paragraph::new("Dashboard..."), frame.area());
        }
    }
}
