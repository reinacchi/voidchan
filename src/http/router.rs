use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{HeaderValue, StatusCode, header},
    response::Response,
    routing::{get, patch, post, put},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    app::state::AppState,
    domain::models::ApiResponse,
    http::handlers::{
        files::{download_file, raw_file, view_file},
        heartbeat::heartbeat,
        presence::{
            delete_presence_kv, get_presence, patch_presence_kv, presence_widget_svg,
            put_presence_kv,
        },
        upload::upload_file,
    },
};

async fn favicon() -> Response {
    let favicon = include_bytes!("../../assets/favicon.ico");

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

pub fn build_router(state: AppState, _max_upload_bytes: usize) -> Router {
    Router::new()
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
        .layer(DefaultBodyLimit::disable())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .fallback(|| async {
            let response = ApiResponse {
                code: StatusCode::NOT_FOUND.as_u16(),
                message: "Route not found.".to_string(),
            };

            (StatusCode::NOT_FOUND, axum::Json(response))
        })
        .with_state(state)
}
