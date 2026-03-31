use ratatui::Frame;
use ratatui::widgets::Paragraph;

use crate::app::{App, AppScreen};

pub fn render(app: &App, frame: &mut Frame) {
    match app.screen {
        AppScreen::Login => {
            frame.render_widget(Paragraph::new("Login screen..."), frame.area());
        }
        AppScreen::Dashboard => {
            frame.render_widget(Paragraph::new("Dashboard..."), frame.area());
        }
    }
}
