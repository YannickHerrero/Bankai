mod dashboard;
mod login;

use ratatui::Frame;

use crate::app::{App, AppScreen};

pub fn render(app: &App, frame: &mut Frame) {
    match app.screen {
        AppScreen::Login => login::render(app, frame),
        AppScreen::Authenticated => dashboard::render(app, frame),
    }
}
