pub enum AppScreen {
    Login,
    Dashboard,
}

pub enum LoginState {
    Prompt,
    WaitingForToken { auth_url: String },
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
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
