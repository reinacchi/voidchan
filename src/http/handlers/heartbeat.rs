use std::time::Instant;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::{app::state::AppState, app::error::AppError};

#[derive(Serialize)]
pub struct HeartbeatResponse {
    pub code: u16,
    pub status: &'static str,
    pub database: &'static str,
    pub latency_ms: u128,
}

pub async fn heartbeat(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HeartbeatResponse>), AppError> {
    let started_at = Instant::now();

    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|err| AppError::Internal(format!("database heartbeat failed: {err}")))?;

    Ok((
        StatusCode::OK,
        Json(HeartbeatResponse {
            code: StatusCode::OK.as_u16(),
            status: "ok",
            database: "reachable",
            latency_ms: started_at.elapsed().as_millis(),
        }),
    ))
}
