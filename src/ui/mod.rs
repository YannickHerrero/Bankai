mod login;
mod page_selector;
mod pages;

use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::Frame;

use crate::app::{App, AppScreen};

pub(crate) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}

pub fn render(app: &App, frame: &mut Frame) {
    match app.screen {
        AppScreen::Login => login::render(app, frame),
        AppScreen::Authenticated => {
            pages::render(app, frame);
            if let Some(ref selector) = app.page_selector {
                page_selector::render(selector, frame);
            }
        }
    }
}
