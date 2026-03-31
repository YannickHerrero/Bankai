mod api;
mod app;
mod auth;
mod token;
mod ui;

use std::time::Duration;

use app::{App, AppScreen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use tokio::sync::mpsc;

enum AppMessage {
    AuthSuccess { token: String, username: String },
    AuthError(String),
    ViewerLoaded(String),
    ViewerError(String),
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let mut app = App::new();
    let (tx, mut rx) = mpsc::unbounded_channel::<AppMessage>();

    if let Ok(config) = token::load_config() {
        if let Some(saved_token) = config.access_token {
            app.token = Some(saved_token.clone());
            app.screen = AppScreen::Dashboard;
            app.loading = true;

            let tx = tx.clone();
            tokio::spawn(async move {
                let client = api::AniListClient::new(saved_token);
                match client.get_viewer().await {
                    Ok(viewer) => {
                        let _ = tx.send(AppMessage::ViewerLoaded(viewer.name));
                    }
                    Err(e) => {
                        let _ = tx.send(AppMessage::ViewerError(e.to_string()));
                    }
                }
            });
        }
    }

    let mut terminal = ratatui::init();

    while app.running {
        terminal
            .draw(|frame| ui::render(&app, frame))
            .expect("failed to draw frame");

        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::AuthSuccess { token, username } => {
                    app.token = Some(token);
                    app.username = Some(username);
                    app.screen = AppScreen::Dashboard;
                    app.loading = false;
                    app.status_message = None;
                }
                AppMessage::AuthError(err) => {
                    app.screen = AppScreen::Login;
                    app.loading = false;
                    app.status_message = Some(err);
                }
                AppMessage::ViewerLoaded(username) => {
                    app.username = Some(username);
                    app.loading = false;
                }
                AppMessage::ViewerError(err) => {
                    app.token = None;
                    app.loading = false;
                    app.screen = AppScreen::Login;
                    app.status_message = Some(format!("Session expired: {err}"));

                    let _ = token::save_config(&token::Config {
                        access_token: None,
                    });
                }
            }
        }

        if event::poll(Duration::from_millis(16)).expect("failed to poll events") {
            if let Event::Key(key) = event::read().expect("failed to read event") {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Enter => {
                        if matches!(app.screen, AppScreen::Login) && !app.loading {
                            let tx = tx.clone();
                            app.loading = true;
                            app.status_message = Some("Opening browser...".into());

                            tokio::spawn(async move {
                                let oauth_config = match auth::OAuthConfig::from_env() {
                                    Ok(c) => c,
                                    Err(e) => {
                                        let _ = tx.send(AppMessage::AuthError(e.to_string()));
                                        return;
                                    }
                                };

                                let token_response = match auth::authenticate(&oauth_config).await
                                {
                                    Ok(t) => t,
                                    Err(e) => {
                                        let _ = tx.send(AppMessage::AuthError(e.to_string()));
                                        return;
                                    }
                                };

                                let _ = token::save_config(&token::Config {
                                    access_token: Some(token_response.access_token.clone()),
                                });

                                let client =
                                    api::AniListClient::new(token_response.access_token.clone());
                                match client.get_viewer().await {
                                    Ok(viewer) => {
                                        let _ = tx.send(AppMessage::AuthSuccess {
                                            token: token_response.access_token,
                                            username: viewer.name,
                                        });
                                    }
                                    Err(e) => {
                                        let _ = tx.send(AppMessage::AuthError(e.to_string()));
                                    }
                                }
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
}
