mod app_state;
mod config;
mod discord;
mod error;
mod models;
mod presence;
mod routes;
mod utils;

use std::net::SocketAddr;

use app_state::AppState;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::Response,
    routing::{get, patch, post, put},
};
use config::Config;
use routes::{
    files::{download_file, raw_file, view_file},
    heartbeat::heartbeat,
    presence::{
        delete_presence_kv, get_presence, patch_presence_kv, presence_widget_svg, put_presence_kv,
    },
    upload::upload_file,
};
use sqlx::mysql::MySqlPoolOptions;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::models::ApiResponse;

async fn favicon() -> Response {
    let favicon = include_bytes!("../assets/favicon.ico");

    let mut response = Response::new(Body::from(favicon.as_slice()));
    *response.status_mut() = StatusCode::OK;

    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/x-icon"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400"),
    );

    response
}

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
        presence: presence::PresenceService::new(),
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

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                let version = env!("CARGO_PKG_VERSION");
                let response = ApiResponse {
                    code: StatusCode::OK.as_u16(),
                    message: format!(
                        "Welcome to the VoidChan API v{}. Join the Discord to get started: https://discord.gg/CqBf9vkD8m",
                        version
                    ),
                };
                axum::Json(response)
            }),
        )
        .route("/favicon.ico", get(favicon))
        .route("/api/providers/sharex", post(upload_file))
        .route("/api/discord/{discord_user_id}", get(get_presence))
        .route(
            "/api/discord/{discord_user_id}/kv",
            patch(patch_presence_kv),
        )
        .route(
            "/api/discord/{discord_user_id}/kv/{key}",
            put(put_presence_kv).delete(delete_presence_kv),
        )
        .route(
            "/api/discord/{discord_user_id}/widget.svg",
            get(presence_widget_svg),
        )
        .route("/heartbeat", get(heartbeat))
        .route("/u/{id}", get(raw_file))
        .route("/download/{id}", get(download_file))
        .route("/v/{id}", get(view_file))
        .layer(RequestBodyLimitLayer::new(max_upload_bytes))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .fallback(|| async {
            let response = ApiResponse {
                code: StatusCode::NOT_FOUND.as_u16(),
                message: "Route not found.".to_string(),
            };

            (StatusCode::NOT_FOUND, axum::Json(response))
        })
        .with_state(state);

    let ipv6 = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], config.app_port));
    let listener = TcpListener::bind(&ipv6).await.unwrap();

    println!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("server failed");
}
