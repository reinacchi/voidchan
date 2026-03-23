use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub app_port: u16,
    pub base_url: String,

    pub database_url: String,

    pub r2_bucket: String,
    pub r2_region: String,
    pub r2_endpoint: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub max_upload_size_mb: usize,

    pub discord_token: Option<String>,
    pub discord_guild_id: Option<u64>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            app_port: env::var("APP_PORT")
                .unwrap_or("3000".into())
                .parse()
                .expect("APP_PORT must be a valid port number"),
            base_url: env::var("BASE_URL").expect("BASE_URL is required"),

            database_url: env::var("DATABASE_URL").expect("DATABASE_URL is required"),

            r2_bucket: env::var("R2_BUCKET").expect("R2_BUCKET is required"),
            r2_region: env::var("R2_REGION").unwrap_or_else(|_| "auto".to_string()),
            r2_endpoint: env::var("R2_ENDPOINT").expect("R2_ENDPOINT is required"),
            r2_access_key_id: env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID is required"),
            r2_secret_access_key: env::var("R2_SECRET_ACCESS_KEY")
                .expect("R2_SECRET_ACCESS_KEY is required"),
            max_upload_size_mb: env::var("MAX_UPLOAD_SIZE_MB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(64),

            discord_token: env::var("DISCORD_TOKEN")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            discord_guild_id: env::var("DISCORD_GUILD_ID")
                .ok()
                .and_then(|v| v.parse().ok()),
        }
    }
}
