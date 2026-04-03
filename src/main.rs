mod api;
mod app;
mod auth;
mod token;
mod ui;

use std::time::Duration;

use api::{ListActivity, MediaListEntry};
use app::{App, AppScreen, DashboardSection, Direction, LoginState, Page};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use tokio::sync::mpsc;

enum AppMessage {
    AuthSuccess {
        token: String,
        username: String,
        user_id: i64,
    },
    AuthError(String),
    ViewerLoaded {
        username: String,
        user_id: i64,
    },
    ViewerError(String),
    WatchingListLoaded(Vec<MediaListEntry>),
    ActivityLoaded(Vec<ListActivity>),
    DataError(String),
}

fn spawn_data_fetches(tx: &mpsc::UnboundedSender<AppMessage>, token: String, user_id: i64) {
    let tx_watch = tx.clone();
    let token_watch = token.clone();
    tokio::spawn(async move {
        let client = api::AniListClient::new(token_watch);
        match client.get_watching_list(user_id).await {
            Ok(list) => {
                let _ = tx_watch.send(AppMessage::WatchingListLoaded(list));
            }
            Err(e) => {
                let _ = tx_watch.send(AppMessage::DataError(e.to_string()));
            }
        }
    });

    let tx_activity = tx.clone();
    tokio::spawn(async move {
        let client = api::AniListClient::new(token);
        match client.get_recent_activity(user_id).await {
            Ok(activities) => {
                let _ = tx_activity.send(AppMessage::ActivityLoaded(activities));
            }
            Err(e) => {
                let _ = tx_activity.send(AppMessage::DataError(e.to_string()));
            }
        }
    });
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    let mut app = App::new();
    let (tx, mut rx) = mpsc::unbounded_channel::<AppMessage>();

    if let Ok(config) = token::load_config() {
        if let Some(saved_token) = config.access_token {
            app.token = Some(saved_token.clone());
            app.screen = AppScreen::Authenticated;
            app.loading = true;

            let tx = tx.clone();
            tokio::spawn(async move {
                let client = api::AniListClient::new(saved_token);
                match client.get_viewer().await {
                    Ok(viewer) => {
                        let _ = tx.send(AppMessage::ViewerLoaded {
                            username: viewer.name,
                            user_id: viewer.id,
                        });
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
                AppMessage::AuthSuccess {
                    token,
                    username,
                    user_id,
                } => {
                    app.token = Some(token.clone());
                    app.username = Some(username);
                    app.user_id = Some(user_id);
                    app.screen = AppScreen::Authenticated;
                    app.loading = false;
                    app.status_message = None;
                    spawn_data_fetches(&tx, token, user_id);
                }
                AppMessage::AuthError(err) => {
                    app.loading = false;
                    app.status_message = Some(err);
                }
                AppMessage::ViewerLoaded { username, user_id } => {
                    app.username = Some(username);
                    app.user_id = Some(user_id);
                    app.loading = false;
                    if let Some(ref token) = app.token {
                        spawn_data_fetches(&tx, token.clone(), user_id);
                    }
                }
                AppMessage::WatchingListLoaded(list) => {
                    app.watching_list = list;
                }
                AppMessage::ActivityLoaded(activities) => {
                    app.recent_activity = activities;
                }
                AppMessage::DataError(err) => {
                    app.status_message = Some(err);
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
                                                user_id: viewer.id,
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
                    (AppScreen::Authenticated, _) => {
                        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                        match key.code {
                            KeyCode::Char('q') => app.quit(),
                            // Panel navigation: Shift+hjkl or Shift+Arrow
                            KeyCode::Char('H') => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Left);
                            }
                            KeyCode::Char('L') => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Right);
                            }
                            KeyCode::Char('J') => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Down);
                            }
                            KeyCode::Char('K') => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Up);
                            }
                            KeyCode::Left if shift => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Left);
                            }
                            KeyCode::Right if shift => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Right);
                            }
                            KeyCode::Down if shift => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Down);
                            }
                            KeyCode::Up if shift => {
                                app.dashboard_section =
                                    app.dashboard_section.navigate(Direction::Up);
                            }
                            // Scroll within section: hjkl or plain arrows
                            KeyCode::Char('j') | KeyCode::Down => {
                                match app.dashboard_section {
                                    DashboardSection::Watching => {
                                        if app.watching_scroll + 1 < app.watching_list.len()
                                        {
                                            app.watching_scroll += 1;
                                        }
                                    }
                                    DashboardSection::Calendar => {
                                        app.calendar_scroll += 1;
                                    }
                                    DashboardSection::Updates => {
                                        if app.updates_scroll + 1
                                            < app.recent_activity.len()
                                        {
                                            app.updates_scroll += 1;
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                match app.dashboard_section {
                                    DashboardSection::Watching => {
                                        app.watching_scroll =
                                            app.watching_scroll.saturating_sub(1);
                                    }
                                    DashboardSection::Calendar => {
                                        app.calendar_scroll =
                                            app.calendar_scroll.saturating_sub(1);
                                    }
                                    DashboardSection::Updates => {
                                        app.updates_scroll =
                                            app.updates_scroll.saturating_sub(1);
                                    }
                                }
                            }
                            _ => {}
                        }
                    },
                }
            }
        }
    }

    ratatui::restore();
}
