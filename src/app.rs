use crate::api::{ListActivity, MediaListEntry};

pub enum AppScreen {
    Login,
    Authenticated,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Search,
    Stats,
}

impl Page {
    pub const ALL: &[Page] = &[Page::Dashboard, Page::Search, Page::Stats];

    pub fn label(&self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Search => "Search",
            Page::Stats => "Stats",
        }
    }
}

pub enum LoginState {
    Prompt,
    WaitingForToken { auth_url: String },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DashboardSection {
    Watching,
    Calendar,
    Updates,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl DashboardSection {
    pub fn navigate(self, direction: Direction) -> Self {
        match (self, direction) {
            (Self::Watching, Direction::Right) => Self::Calendar,
            (Self::Calendar, Direction::Left) => Self::Watching,
            (Self::Calendar, Direction::Down) => Self::Updates,
            (Self::Updates, Direction::Left) => Self::Watching,
            (Self::Updates, Direction::Up) => Self::Calendar,
            _ => self,
        }
    }
}

pub struct App {
    pub screen: AppScreen,
    pub running: bool,
    pub username: Option<String>,
    pub token: Option<String>,
    pub status_message: Option<String>,
    pub loading: bool,
    pub login_state: LoginState,
    pub token_input: String,
    pub user_id: Option<i64>,
    pub watching_list: Vec<MediaListEntry>,
    pub recent_activity: Vec<ListActivity>,
    pub dashboard_section: DashboardSection,
    pub watching_scroll: usize,
    pub updates_scroll: usize,
    pub calendar_scroll: usize,
    pub page: Page,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: AppScreen::Login,
            running: true,
            username: None,
            token: None,
            status_message: None,
            loading: false,
            login_state: LoginState::Prompt,
            token_input: String::new(),
            user_id: None,
            watching_list: Vec::new(),
            recent_activity: Vec::new(),
            dashboard_section: DashboardSection::Watching,
            watching_scroll: 0,
            updates_scroll: 0,
            calendar_scroll: 0,
            page: Page::Dashboard,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
