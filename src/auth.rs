use std::fmt;

use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const AUTH_URL: &str = "https://anilist.co/api/v2/oauth/authorize";
const TOKEN_URL: &str = "https://anilist.co/api/v2/oauth/token";
const REDIRECT_URI: &str = "http://localhost:8910/callback";

pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl OAuthConfig {
    pub fn from_env() -> Result<Self, AuthError> {
        let client_id = std::env::var("BANKAI_CLIENT_ID")
            .map_err(|_| AuthError::MissingEnv("BANKAI_CLIENT_ID"))?;
        let client_secret = std::env::var("BANKAI_CLIENT_SECRET")
            .map_err(|_| AuthError::MissingEnv("BANKAI_CLIENT_SECRET"))?;
        Ok(Self {
            client_id,
            client_secret,
        })
    }
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Debug)]
pub enum AuthError {
    MissingEnv(&'static str),
    BrowserOpen(std::io::Error),
    ServerBind(std::io::Error),
    CallbackRead(std::io::Error),
    CallbackParse(String),
    TokenExchange(reqwest::Error),
    TokenDeserialize(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv(var) => write!(f, "missing environment variable: {var}"),
            Self::BrowserOpen(e) => write!(f, "failed to open browser: {e}"),
            Self::ServerBind(e) => write!(f, "failed to bind callback server: {e}"),
            Self::CallbackRead(e) => write!(f, "failed to read callback: {e}"),
            Self::CallbackParse(msg) => write!(f, "failed to parse callback: {msg}"),
            Self::TokenExchange(e) => write!(f, "token exchange failed: {e}"),
            Self::TokenDeserialize(msg) => write!(f, "failed to deserialize token: {msg}"),
        }
    }
}

pub async fn authenticate(config: &OAuthConfig) -> Result<TokenResponse, AuthError> {
    let auth_url = format!(
        "{AUTH_URL}?client_id={}&redirect_uri={REDIRECT_URI}&response_type=code",
        config.client_id
    );

    open::that(&auth_url).map_err(AuthError::BrowserOpen)?;

    let listener = TcpListener::bind("127.0.0.1:8910")
        .await
        .map_err(AuthError::ServerBind)?;

    let (mut stream, _) = listener.accept().await.map_err(AuthError::CallbackRead)?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.map_err(AuthError::CallbackRead)?;
    let request = String::from_utf8_lossy(&buf[..n]);

    let code = extract_code(&request)?;

    let response_html = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><h1>Success!</h1><p>You can close this window and return to bankai.</p></body></html>";
    let _ = stream.write_all(response_html.as_bytes()).await;
    let _ = stream.shutdown().await;

    drop(listener);

    exchange_code(config, &code).await
}

fn extract_code(request: &str) -> Result<String, AuthError> {
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| AuthError::CallbackParse("empty request".into()))?;

    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AuthError::CallbackParse("no path in request".into()))?;

    let query = path
        .split('?')
        .nth(1)
        .ok_or_else(|| AuthError::CallbackParse("no query string".into()))?;

    for param in query.split('&') {
        if let Some(value) = param.strip_prefix("code=") {
            return Ok(value.to_string());
        }
    }

    Err(AuthError::CallbackParse("no code parameter found".into()))
}

async fn exchange_code(config: &OAuthConfig, code: &str) -> Result<TokenResponse, AuthError> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": config.client_id,
        "client_secret": config.client_secret,
        "redirect_uri": REDIRECT_URI,
        "code": code,
    });

    let response = client
        .post(TOKEN_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(AuthError::TokenExchange)?;

    let text = response.text().await.map_err(AuthError::TokenExchange)?;

    serde_json::from_str::<TokenResponse>(&text)
        .map_err(|e| AuthError::TokenDeserialize(format!("{e}: {text}")))
}
