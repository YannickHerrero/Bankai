use std::fmt;

const AUTH_URL: &str = "https://anilist.co/api/v2/oauth/authorize";

#[derive(Debug)]
pub enum AuthError {
    MissingEnv(&'static str),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv(var) => write!(f, "missing environment variable: {var}"),
        }
    }
}

pub fn build_auth_url() -> Result<String, AuthError> {
    let client_id =
        std::env::var("BANKAI_CLIENT_ID").map_err(|_| AuthError::MissingEnv("BANKAI_CLIENT_ID"))?;

    Ok(format!(
        "{AUTH_URL}?client_id={client_id}&response_type=token"
    ))
}
