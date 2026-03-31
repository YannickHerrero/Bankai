use std::fmt;

use serde::Deserialize;

const AUTH_URL: &str = "https://anilist.co/api/v2/oauth/authorize";
const TOKEN_URL: &str = "https://anilist.co/api/v2/oauth/token";
const REDIRECT_URI: &str = "https://anilist.co/api/v2/oauth/pin";

#[derive(Debug)]
pub enum AuthError {
    MissingEnv(&'static str),
    TokenExchange(reqwest::Error),
    TokenDeserialize(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv(var) => write!(f, "missing environment variable: {var}"),
            Self::TokenExchange(e) => write!(f, "token exchange failed: {e}"),
            Self::TokenDeserialize(msg) => write!(f, "failed to deserialize token: {msg}"),
        }
    }
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
}

pub fn build_auth_url() -> Result<String, AuthError> {
    let client_id =
        std::env::var("BANKAI_CLIENT_ID").map_err(|_| AuthError::MissingEnv("BANKAI_CLIENT_ID"))?;

    Ok(format!(
        "{AUTH_URL}?client_id={client_id}&redirect_uri={REDIRECT_URI}&response_type=code"
    ))
}

pub async fn exchange_code(code: &str) -> Result<TokenResponse, AuthError> {
    let client_id =
        std::env::var("BANKAI_CLIENT_ID").map_err(|_| AuthError::MissingEnv("BANKAI_CLIENT_ID"))?;
    let client_secret = std::env::var("BANKAI_CLIENT_SECRET")
        .map_err(|_| AuthError::MissingEnv("BANKAI_CLIENT_SECRET"))?;

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": client_id,
        "client_secret": client_secret,
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
