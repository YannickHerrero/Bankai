pub enum AppScreen {
    Login,
    Dashboard,
}

pub struct App {
    pub screen: AppScreen,
    pub running: bool,
    pub username: Option<String>,
    pub token: Option<String>,
    pub status_message: Option<String>,
    pub loading: bool,
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
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
