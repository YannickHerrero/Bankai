mod dashboard;
mod search;
mod stats;

use ratatui::Frame;

use crate::app::{App, Page};

pub fn render(app: &App, frame: &mut Frame) {
    match app.page {
        Page::Dashboard => dashboard::render(app, frame),
        Page::Search => search::render(app, frame),
        Page::Stats => stats::render(app, frame),
    }
}
