mod api;
mod app;
mod auth;
mod token;
mod ui;

use std::time::Duration;

use app::{App, AppScreen, LoginState};
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
                    app.login_state = LoginState::Prompt;
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

                match (&app.screen, &app.login_state) {
                    (AppScreen::Login, LoginState::Prompt) => match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Enter => match auth::build_auth_url() {
                            Ok(url) => {
                                app.login_state = LoginState::WaitingForToken { auth_url: url };
                                app.status_message = None;
                            }
                            Err(e) => {
                                app.status_message = Some(e.to_string());
                            }
                        },
                        _ => {}
                    },
                    (AppScreen::Login, LoginState::WaitingForToken { .. }) => match key.code {
                        KeyCode::Char(c) => {
                            app.token_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.token_input.pop();
                        }
                        KeyCode::Esc => {
                            app.login_state = LoginState::Prompt;
                            app.token_input.clear();
                            app.status_message = None;
                        }
                        KeyCode::Enter => {
                            let code = app.token_input.trim().to_string();
                            if code.is_empty() {
                                app.status_message = Some("Code cannot be empty".into());
                            } else {
                                app.loading = true;
                                app.status_message = Some("Exchanging code for token...".into());
                                app.token_input.clear();

                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    let token_response = match auth::exchange_code(&code).await {
                                        Ok(t) => t,
                                        Err(e) => {
                                            let _ = tx.send(AppMessage::AuthError(e.to_string()));
                                            return;
                                        }
                                    };

                                    let access_token = token_response.access_token;

                                    let _ = token::save_config(&token::Config {
                                        access_token: Some(access_token.clone()),
                                    });

                                    let client = api::AniListClient::new(access_token.clone());
                                    match client.get_viewer().await {
                                        Ok(viewer) => {
                                            let _ = tx.send(AppMessage::AuthSuccess {
                                                token: access_token,
                                                username: viewer.name,
                                            });
                                        }
                                        Err(e) => {
                                            let _ = token::save_config(&token::Config {
                                                access_token: None,
                                            });
                                            let _ = tx.send(AppMessage::AuthError(format!(
                                                "Invalid token: {e}"
                                            )));
                                        }
                                    }
                                });
                            }
                        }
                        _ => {}
                    },
                    (AppScreen::Dashboard, _) => match key.code {
                        KeyCode::Char('q') => app.quit(),
                        _ => {}
                    },
                }
            }
        }
    }

    ratatui::restore();
}
