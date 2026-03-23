use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub api_token: String,
    pub discord_user_id: Option<String>,
    pub preferred_url_mode: String,
    pub preferred_hex_colour: String,
    pub is_blacklisted: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct StoredFile {
    pub id: String,
    pub original_name: Option<String>,
    pub object_key: String,
    pub mime_type: String,
    pub extension: String,
    pub size: u64,
    pub nsfw: bool,
    pub uploader: String,
    pub uploader_id: Option<u64>,
    pub preferred_hex_colour: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub code: u16,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub code: u16,
    pub message: String,
}
