use std::fmt;

use serde::Deserialize;

const ANILIST_GRAPHQL_URL: &str = "https://graphql.anilist.co";

// --- Domain models ---

#[derive(Deserialize, Clone, Debug)]
pub struct MediaTitle {
    pub romaji: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NextAiringEpisode {
    #[serde(rename = "airingAt")]
    pub airing_at: i64,
    pub episode: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Media {
    pub id: i64,
    pub title: MediaTitle,
    pub episodes: Option<i32>,
    #[serde(rename = "nextAiringEpisode")]
    pub next_airing_episode: Option<NextAiringEpisode>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MediaListEntry {
    pub media: Media,
    pub progress: i32,
    pub score: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ListActivity {
    pub status: String,
    pub progress: Option<String>,
    pub media: Media,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

// --- API response types ---

#[derive(Deserialize)]
pub struct Viewer {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
struct ViewerData {
    #[serde(rename = "Viewer")]
    viewer: Viewer,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: ViewerData,
}

pub struct AniListClient {
    http: reqwest::Client,
    token: String,
}

#[derive(Debug)]
pub enum ApiError {
    Network(reqwest::Error),
    Deserialize(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(e) => write!(f, "network error: {e}"),
            Self::Deserialize(msg) => write!(f, "deserialization error: {msg}"),
        }
    }
}

impl AniListClient {
    pub fn new(token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            token,
        }
    }

    pub async fn get_viewer(&self) -> Result<Viewer, ApiError> {
        let body = serde_json::json!({
            "query": "query { Viewer { id name } }"
        });

        let response = self
            .http
            .post(ANILIST_GRAPHQL_URL)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(ApiError::Network)?;

        let text = response.text().await.map_err(ApiError::Network)?;

        let parsed: GraphQLResponse = serde_json::from_str(&text)
            .map_err(|e| ApiError::Deserialize(format!("{e}: {text}")))?;

        Ok(parsed.data.viewer)
    }
}
