mod api;
mod app;
mod auth;
mod token;
mod ui;

use std::time::Duration;

use api::{ListActivity, MediaListEntry, MediaListStatus, SearchMedia, UserMediaListEntry};
use app::{
    App, AppScreen, DashboardSection, Direction, LoginState, Page, PageSelectorState, SearchFocus,
    SearchPopup,
};
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
    SearchResults(Vec<SearchMedia>),
    SearchError(String),
    MediaSaved {
        media_id: i64,
        entry_id: i64,
        status: String,
    },
    MediaDeleted {
        media_id: i64,
    },
    MediaSaveError(String),
    MediaDeleteError(String),
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
                AppMessage::SearchResults(results) => {
                    app.search.searching = false;
                    app.search.result_scroll = 0;
                    app.search.detail_scroll = 0;
                    if !results.is_empty() {
                        app.search.focus = SearchFocus::Results;
                    }
                    app.search.results = results;
                }
                AppMessage::SearchError(err) => {
                    app.search.searching = false;
                    app.status_message = Some(err);
                }
                AppMessage::MediaSaved {
                    media_id,
                    entry_id,
                    status,
                } => {
                    if let Some(media) = app.search.results.iter_mut().find(|m| m.id == media_id) {
                        let prev = media.media_list_entry.take();
                        let progress = prev.as_ref().map(|e| e.progress).unwrap_or(0);
                        let score = prev.as_ref().map(|e| e.score).unwrap_or(0.0);
                        media.media_list_entry = Some(UserMediaListEntry {
                            id: entry_id,
                            status: status.clone(),
                            progress,
                            score,
                        });
                    }
                    app.search.popup = None;
                    app.status_message = Some(format!("Added to list as {status}"));
                }
                AppMessage::MediaDeleted { media_id } => {
                    if let Some(media) = app.search.results.iter_mut().find(|m| m.id == media_id) {
                        media.media_list_entry = None;
                    }
                    app.search.popup = None;
                    app.status_message = Some("Removed from list".into());
                }
                AppMessage::MediaSaveError(err) => {
                    app.search.popup = None;
                    app.status_message = Some(format!("Save failed: {err}"));
                }
                AppMessage::MediaDeleteError(err) => {
                    app.search.popup = None;
                    app.status_message = Some(format!("Delete failed: {err}"));
                }
            }
        }

        if event::poll(Duration::from_millis(16)).expect("failed to poll events") {
            if let Event::Key(key) = event::read().expect("failed to read event") {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Page selector popup captures all input when open
                if let Some(ref mut selector) = app.page_selector {
                    match key.code {
                        KeyCode::Esc => {
                            app.page_selector = None;
                        }
                        KeyCode::Enter => {
                            if let Some(page) = selector.selected_page() {
                                app.page = page;
                            }
                            app.page_selector = None;
                        }
                        KeyCode::Up => {
                            selector.move_up();
                        }
                        KeyCode::Down | KeyCode::Tab => {
                            selector.move_down();
                        }
                        KeyCode::Char(c) => {
                            selector.query.push(c);
                            selector.update_filter();
                        }
                        KeyCode::Backspace => {
                            selector.query.pop();
                            selector.update_filter();
                        }
                        _ => {}
                    }
                    continue;
                }

                // Space opens page selector on authenticated screens
                // (but not when typing in the search input)
                if matches!(app.screen, AppScreen::Authenticated)
                    && key.code == KeyCode::Char(' ')
                    && !(app.page == Page::Search && app.search.focus == SearchFocus::Input)
                {
                    app.page_selector = Some(PageSelectorState::new());
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
                    (AppScreen::Authenticated, _) => match app.page {
                        Page::Dashboard => {
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
                        }
                        Page::Search => {
                            // Popup captures all input when active
                            if let Some(ref popup) = app.search.popup {
                                match popup {
                                    SearchPopup::StatusPicker { selected } => {
                                        let mut sel = *selected;
                                        match key.code {
                                            KeyCode::Esc => {
                                                app.search.popup = None;
                                            }
                                            KeyCode::Char('j') | KeyCode::Down => {
                                                sel = (sel + 1) % MediaListStatus::ALL.len();
                                                app.search.popup =
                                                    Some(SearchPopup::StatusPicker { selected: sel });
                                            }
                                            KeyCode::Char('k') | KeyCode::Up => {
                                                sel = if sel == 0 {
                                                    MediaListStatus::ALL.len() - 1
                                                } else {
                                                    sel - 1
                                                };
                                                app.search.popup =
                                                    Some(SearchPopup::StatusPicker { selected: sel });
                                            }
                                            KeyCode::Enter => {
                                                let status = MediaListStatus::ALL[sel];
                                                if let Some(media) = app.search.selected_media() {
                                                    let media_id = media.id;
                                                    let status_str =
                                                        status.api_value().to_string();
                                                    let token = app.token.clone().unwrap();
                                                    let tx = tx.clone();
                                                    tokio::spawn(async move {
                                                        let client =
                                                            api::AniListClient::new(token);
                                                        match client
                                                            .save_media_list_entry(
                                                                media_id,
                                                                &status_str,
                                                            )
                                                            .await
                                                        {
                                                            Ok((entry_id, saved_status)) => {
                                                                let _ = tx.send(
                                                                    AppMessage::MediaSaved {
                                                                        media_id,
                                                                        entry_id,
                                                                        status: saved_status,
                                                                    },
                                                                );
                                                            }
                                                            Err(e) => {
                                                                let _ = tx.send(
                                                                    AppMessage::MediaSaveError(
                                                                        e.to_string(),
                                                                    ),
                                                                );
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    SearchPopup::RemoveConfirm { confirm_selected } => {
                                        let mut confirm = *confirm_selected;
                                        match key.code {
                                            KeyCode::Esc | KeyCode::Char('n') => {
                                                app.search.popup = None;
                                            }
                                            KeyCode::Enter if confirm => {
                                                if let Some(media) = app.search.selected_media() {
                                                    if let Some(ref entry) =
                                                        media.media_list_entry
                                                    {
                                                        let entry_id = entry.id;
                                                        let media_id = media.id;
                                                        let token = app.token.clone().unwrap();
                                                        let tx = tx.clone();
                                                        tokio::spawn(async move {
                                                            let client =
                                                                api::AniListClient::new(token);
                                                            match client
                                                                .delete_media_list_entry(entry_id)
                                                                .await
                                                            {
                                                                Ok(_) => {
                                                                    let _ = tx.send(
                                                                        AppMessage::MediaDeleted {
                                                                            media_id,
                                                                        },
                                                                    );
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.send(
                                                                        AppMessage::MediaDeleteError(
                                                                            e.to_string(),
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                            KeyCode::Enter => {
                                                app.search.popup = None;
                                            }
                                            KeyCode::Left
                                            | KeyCode::Right
                                            | KeyCode::Char('h')
                                            | KeyCode::Char('l')
                                            | KeyCode::Tab => {
                                                confirm = !confirm;
                                                app.search.popup =
                                                    Some(SearchPopup::RemoveConfirm {
                                                        confirm_selected: confirm,
                                                    });
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            } else {
                                match app.search.focus {
                                    SearchFocus::Input => match key.code {
                                        KeyCode::Char(c) => {
                                            app.search.query.push(c);
                                        }
                                        KeyCode::Backspace => {
                                            app.search.query.pop();
                                        }
                                        KeyCode::Tab => {
                                            app.search.media_type =
                                                app.search.media_type.toggle();
                                        }
                                        KeyCode::Enter => {
                                            if !app.search.query.is_empty() {
                                                app.search.searching = true;
                                                let query = app.search.query.clone();
                                                let media_type = app
                                                    .search
                                                    .media_type
                                                    .api_value()
                                                    .to_string();
                                                let user_id = app.user_id.unwrap_or(0);
                                                let token = app.token.clone().unwrap();
                                                let tx = tx.clone();
                                                tokio::spawn(async move {
                                                    let client = api::AniListClient::new(token);
                                                    match client
                                                        .search_media(
                                                            &query,
                                                            &media_type,
                                                            user_id,
                                                        )
                                                        .await
                                                    {
                                                        Ok(results) => {
                                                            let _ = tx.send(
                                                                AppMessage::SearchResults(results),
                                                            );
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(
                                                                AppMessage::SearchError(
                                                                    e.to_string(),
                                                                ),
                                                            );
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        KeyCode::Down => {
                                            if !app.search.results.is_empty() {
                                                app.search.focus = SearchFocus::Results;
                                            }
                                        }
                                        KeyCode::Esc => {
                                            if !app.search.results.is_empty() {
                                                app.search.focus = SearchFocus::Results;
                                            }
                                        }
                                        _ => {}
                                    },
                                    SearchFocus::Results => match key.code {
                                        KeyCode::Char('q') => app.quit(),
                                        KeyCode::Char('j') | KeyCode::Down => {
                                            if app.search.result_scroll + 1
                                                < app.search.results.len()
                                            {
                                                app.search.result_scroll += 1;
                                                app.search.detail_scroll = 0;
                                            }
                                        }
                                        KeyCode::Char('k') | KeyCode::Up => {
                                            app.search.result_scroll =
                                                app.search.result_scroll.saturating_sub(1);
                                            app.search.detail_scroll = 0;
                                        }
                                        KeyCode::Enter => {
                                            if app.search.selected_media().is_some() {
                                                app.search.focus = SearchFocus::Detail;
                                                app.search.detail_scroll = 0;
                                            }
                                        }
                                        KeyCode::Esc | KeyCode::Char('/') => {
                                            app.search.focus = SearchFocus::Input;
                                        }
                                        _ => {}
                                    },
                                    SearchFocus::Detail => match key.code {
                                        KeyCode::Char('q') => app.quit(),
                                        KeyCode::Esc => {
                                            app.search.focus = SearchFocus::Results;
                                        }
                                        KeyCode::Char('j') | KeyCode::Down => {
                                            app.search.detail_scroll += 1;
                                        }
                                        KeyCode::Char('k') | KeyCode::Up => {
                                            app.search.detail_scroll =
                                                app.search.detail_scroll.saturating_sub(1);
                                        }
                                        KeyCode::Char('a') => {
                                            if let Some(media) = app.search.selected_media() {
                                                if media.media_list_entry.is_some() {
                                                    app.search.popup =
                                                        Some(SearchPopup::RemoveConfirm {
                                                            confirm_selected: false,
                                                        });
                                                } else {
                                                    app.search.popup =
                                                        Some(SearchPopup::StatusPicker {
                                                            selected: 0,
                                                        });
                                                }
                                            }
                                        }
                                        _ => {}
                                    },
                                }
                            }
                        }
                        Page::Stats => match key.code {
                            KeyCode::Char('q') => app.quit(),
                            _ => {}
                        },
                    },
                }
            }
        }
    }

    ratatui::restore();
}
