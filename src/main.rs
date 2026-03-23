#![allow(dead_code)]

use std::net::SocketAddr;

use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use sqlx::mysql::MySqlPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use voidchan::{
    app::{config::Config, state::AppState},
    http::router::build_router,
    integrations::discord,
    services::presence::PresenceService,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let db = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to MySQL");

    let credentials = Credentials::new(
        config.r2_access_key_id.clone(),
        config.r2_secret_access_key.clone(),
        None,
        None,
        "static",
    );

    let s3_config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(config.r2_region.clone()))
        .endpoint_url(config.r2_endpoint.clone())
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();

    let s3 = aws_sdk_s3::Client::from_conf(s3_config);

    let max_upload_bytes = config.max_upload_size_mb * 1024 * 1024;

    let state = AppState {
        config: config.clone(),
        db,
        s3,
        presence: PresenceService::new(),
    };

    if let (Some(token), Some(guild_id)) = (config.discord_token.clone(), config.discord_guild_id) {
        let discord_state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = discord::run_bot(discord_state, token, guild_id).await {
                tracing::error!("discord bot failed: {err}");
            }
        });
    } else {
        tracing::warn!("Discord bot disabled because DISCORD_TOKEN or DISCORD_GUILD_ID is missing");
    }

    let app = build_router(state, max_upload_bytes);

    let ipv6 = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], config.app_port));
    let listener = TcpListener::bind(&ipv6).await.unwrap();

    println!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("server failed");
}
