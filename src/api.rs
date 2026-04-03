use std::fmt;

use serde::Deserialize;

const ANILIST_GRAPHQL_URL: &str = "https://graphql.anilist.co";

// --- Domain models ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaListStatus {
    Planning,
    Current,
    Completed,
    Paused,
    Dropped,
}

impl MediaListStatus {
    pub const ALL: &[MediaListStatus] = &[
        Self::Planning,
        Self::Current,
        Self::Completed,
        Self::Paused,
        Self::Dropped,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Current => "Watching / Reading",
            Self::Planning => "Plan to Watch / Read",
            Self::Completed => "Completed",
            Self::Paused => "Paused",
            Self::Dropped => "Dropped",
        }
    }

    pub fn api_value(&self) -> &'static str {
        match self {
            Self::Current => "CURRENT",
            Self::Planning => "PLANNING",
            Self::Completed => "COMPLETED",
            Self::Paused => "PAUSED",
            Self::Dropped => "DROPPED",
        }
    }
}

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
    #[allow(dead_code)]
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

// --- Search models ---

#[derive(Deserialize, Clone, Debug)]
pub struct SearchMediaTitle {
    pub romaji: String,
    pub english: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserMediaListEntry {
    pub id: i64,
    pub status: String,
    pub progress: i32,
    pub score: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SearchMedia {
    pub id: i64,
    pub title: SearchMediaTitle,
    pub format: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<i32>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<i32>,
    pub episodes: Option<i32>,
    pub chapters: Option<i32>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub genres: Option<Vec<String>>,
    #[serde(rename = "mediaListEntry")]
    pub media_list_entry: Option<UserMediaListEntry>,
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

#[derive(Deserialize)]
struct MediaListGroup {
    entries: Vec<MediaListEntry>,
}

#[derive(Deserialize)]
struct MediaListCollection {
    lists: Vec<MediaListGroup>,
}

#[derive(Deserialize)]
struct MediaListCollectionData {
    #[serde(rename = "MediaListCollection")]
    media_list_collection: MediaListCollection,
}

#[derive(Deserialize)]
struct MediaListResponse {
    data: MediaListCollectionData,
}

#[derive(Deserialize)]
struct ActivityUnion {
    status: Option<String>,
    progress: Option<String>,
    media: Option<Media>,
    #[serde(rename = "createdAt")]
    created_at: Option<i64>,
}

#[derive(Deserialize)]
struct ActivitiesPage {
    activities: Vec<ActivityUnion>,
}

#[derive(Deserialize)]
struct ActivityPageData {
    #[serde(rename = "Page")]
    page: ActivitiesPage,
}

#[derive(Deserialize)]
struct ActivityResponse {
    data: ActivityPageData,
}

#[derive(Deserialize)]
struct SearchPage {
    media: Vec<SearchMedia>,
}

#[derive(Deserialize)]
struct SearchPageData {
    #[serde(rename = "Page")]
    page: SearchPage,
}

#[derive(Deserialize)]
struct SearchResponse {
    data: SearchPageData,
}

#[derive(Deserialize)]
struct SavedEntry {
    id: i64,
    status: String,
}

#[derive(Deserialize)]
struct SaveMediaListEntryData {
    #[serde(rename = "SaveMediaListEntry")]
    save_media_list_entry: SavedEntry,
}

#[derive(Deserialize)]
struct SaveMediaListResponse {
    data: SaveMediaListEntryData,
}

#[derive(Deserialize)]
struct DeletedResult {
    deleted: bool,
}

#[derive(Deserialize)]
struct DeleteMediaListEntryData {
    #[serde(rename = "DeleteMediaListEntry")]
    delete_media_list_entry: DeletedResult,
}

#[derive(Deserialize)]
struct DeleteMediaListResponse {
    data: DeleteMediaListEntryData,
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

    pub async fn get_watching_list(&self, user_id: i64) -> Result<Vec<MediaListEntry>, ApiError> {
        let query = r#"
            query ($userId: Int) {
                MediaListCollection(userId: $userId, type: ANIME, status: CURRENT) {
                    lists {
                        entries {
                            media {
                                id
                                title { romaji }
                                episodes
                                nextAiringEpisode { airingAt episode }
                            }
                            progress
                            score
                        }
                    }
                }
            }
        "#;

        let body = serde_json::json!({ "query": query, "variables": { "userId": user_id } });

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

        let parsed: MediaListResponse = serde_json::from_str(&text)
            .map_err(|e| ApiError::Deserialize(format!("{e}: {text}")))?;

        let entries = parsed
            .data
            .media_list_collection
            .lists
            .into_iter()
            .flat_map(|group| group.entries)
            .collect();

        Ok(entries)
    }

    pub async fn get_recent_activity(&self, user_id: i64) -> Result<Vec<ListActivity>, ApiError> {
        let query = r#"
            query ($userId: Int) {
                Page(perPage: 20) {
                    activities(userId: $userId, sort: ID_DESC) {
                        ... on ListActivity {
                            status
                            progress
                            media {
                                id
                                title { romaji }
                                episodes
                                nextAiringEpisode { airingAt episode }
                            }
                            createdAt
                        }
                    }
                }
            }
        "#;

        let body = serde_json::json!({ "query": query, "variables": { "userId": user_id } });

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

        let parsed: ActivityResponse = serde_json::from_str(&text)
            .map_err(|e| ApiError::Deserialize(format!("{e}: {text}")))?;

        let activities = parsed
            .data
            .page
            .activities
            .into_iter()
            .filter_map(|a| {
                Some(ListActivity {
                    status: a.status?,
                    progress: a.progress,
                    media: a.media?,
                    created_at: a.created_at?,
                })
            })
            .collect();

        Ok(activities)
    }

    pub async fn search_media(
        &self,
        search: &str,
        media_type: &str,
    ) -> Result<Vec<SearchMedia>, ApiError> {
        let query = r#"
            query ($search: String, $type: MediaType) {
                Page(page: 1, perPage: 20) {
                    media(search: $search, type: $type) {
                        id
                        title { romaji english }
                        format
                        seasonYear
                        averageScore
                        episodes
                        chapters
                        description(asHtml: false)
                        status
                        genres
                        mediaListEntry {
                            id
                            status
                            progress
                            score
                        }
                    }
                }
            }
        "#;

        let body = serde_json::json!({
            "query": query,
            "variables": { "search": search, "type": media_type }
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

        let parsed: SearchResponse = serde_json::from_str(&text)
            .map_err(|e| ApiError::Deserialize(format!("{e}: {text}")))?;

        Ok(parsed.data.page.media)
    }

    pub async fn save_media_list_entry(
        &self,
        media_id: i64,
        status: &str,
    ) -> Result<(i64, String), ApiError> {
        let query = r#"
            mutation ($mediaId: Int, $status: MediaListStatus) {
                SaveMediaListEntry(mediaId: $mediaId, status: $status) {
                    id
                    status
                }
            }
        "#;

        let body = serde_json::json!({
            "query": query,
            "variables": { "mediaId": media_id, "status": status }
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

        let parsed: SaveMediaListResponse = serde_json::from_str(&text)
            .map_err(|e| ApiError::Deserialize(format!("{e}: {text}")))?;

        let entry = parsed.data.save_media_list_entry;
        Ok((entry.id, entry.status))
    }

    pub async fn delete_media_list_entry(&self, entry_id: i64) -> Result<bool, ApiError> {
        let query = r#"
            mutation ($id: Int) {
                DeleteMediaListEntry(id: $id) {
                    deleted
                }
            }
        "#;

        let body = serde_json::json!({
            "query": query,
            "variables": { "id": entry_id }
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

        let parsed: DeleteMediaListResponse = serde_json::from_str(&text)
            .map_err(|e| ApiError::Deserialize(format!("{e}: {text}")))?;

        Ok(parsed.data.delete_media_list_entry.deleted)
    }
}
