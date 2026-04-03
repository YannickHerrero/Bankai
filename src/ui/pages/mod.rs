mod dashboard;

use ratatui::Frame;

use crate::app::{App, Page};

pub fn render(app: &App, frame: &mut Frame) {
    match app.page {
        Page::Dashboard => dashboard::render(app, frame),
        _ => {}
    }
}
