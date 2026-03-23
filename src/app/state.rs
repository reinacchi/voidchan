use aws_sdk_s3::Client as S3Client;
use sqlx::MySqlPool;

use crate::{app::config::Config, services::presence::PresenceService};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: MySqlPool,
    pub s3: S3Client,
    pub presence: PresenceService,
}
