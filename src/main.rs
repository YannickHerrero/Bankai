mod app;
mod ui;

use std::time::Duration;

use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[tokio::main]
async fn main() {
    let mut app = App::new();
    let mut terminal = ratatui::init();

    while app.running {
        terminal
            .draw(|frame| ui::render(&app, frame))
            .expect("failed to draw frame");

        if event::poll(Duration::from_millis(16)).expect("failed to poll events") {
            if let Event::Key(key) = event::read().expect("failed to read event") {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    app.quit();
                }
            }
        }
    }

    ratatui::restore();
}
