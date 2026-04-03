use crate::api::{ListActivity, MediaListEntry, SearchMedia, UserStatistics};

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchMediaType {
    Anime,
    Manga,
}

impl SearchMediaType {
    pub fn api_value(&self) -> &'static str {
        match self {
            Self::Anime => "ANIME",
            Self::Manga => "MANGA",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Self::Anime => Self::Manga,
            Self::Manga => Self::Anime,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchFocus {
    Input,
    Results,
    Detail,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchPopup {
    StatusPicker { selected: usize },
    RemoveConfirm { confirm_selected: bool },
}

pub struct SearchState {
    pub query: String,
    pub media_type: SearchMediaType,
    pub focus: SearchFocus,
    pub results: Vec<SearchMedia>,
    pub result_scroll: usize,
    pub detail_scroll: usize,
    pub searching: bool,
    pub popup: Option<SearchPopup>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            media_type: SearchMediaType::Anime,
            focus: SearchFocus::Input,
            results: Vec::new(),
            result_scroll: 0,
            detail_scroll: 0,
            searching: false,
            popup: None,
        }
    }

    pub fn selected_media(&self) -> Option<&SearchMedia> {
        self.results.get(self.result_scroll)
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatsSection {
    Overview,
    ScoreDistribution,
    TopGenres,
    Formats,
}

impl StatsSection {
    pub fn navigate(self, direction: Direction) -> Self {
        match (self, direction) {
            (Self::Overview, Direction::Right) => Self::ScoreDistribution,
            (Self::Overview, Direction::Down) => Self::TopGenres,
            (Self::ScoreDistribution, Direction::Left) => Self::Overview,
            (Self::ScoreDistribution, Direction::Down) => Self::Formats,
            (Self::TopGenres, Direction::Right) => Self::Formats,
            (Self::TopGenres, Direction::Up) => Self::Overview,
            (Self::Formats, Direction::Left) => Self::TopGenres,
            (Self::Formats, Direction::Up) => Self::ScoreDistribution,
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
    pub page_selector: Option<PageSelectorState>,
    pub search: SearchState,
    pub stats_data: Option<UserStatistics>,
    pub stats_section: StatsSection,
    pub stats_overview_scroll: usize,
    pub stats_genres_scroll: usize,
    pub stats_formats_scroll: usize,
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
            page_selector: None,
            search: SearchState::new(),
            stats_data: None,
            stats_section: StatsSection::Overview,
            stats_overview_scroll: 0,
            stats_genres_scroll: 0,
            stats_formats_scroll: 0,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

pub struct PageSelectorState {
    pub query: String,
    pub selected: usize,
    pub filtered: Vec<Page>,
}

impl PageSelectorState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            selected: 0,
            filtered: Page::ALL.to_vec(),
        }
    }

    pub fn update_filter(&mut self) {
        self.filtered = Page::ALL
            .iter()
            .copied()
            .filter(|p| fuzzy_matches(&self.query, p.label()))
            .collect();
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn move_up(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = if self.selected == 0 {
                self.filtered.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1) % self.filtered.len();
        }
    }

    pub fn selected_page(&self) -> Option<Page> {
        self.filtered.get(self.selected).copied()
    }
}

fn fuzzy_matches(query: &str, haystack: &str) -> bool {
    let mut haystack_chars = haystack.chars().flat_map(|c| c.to_lowercase());
    for qc in query.chars().flat_map(|c| c.to_lowercase()) {
        if haystack_chars.find(|&hc| hc == qc).is_none() {
            return false;
        }
    }
    true
}
