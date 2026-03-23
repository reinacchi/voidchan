use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse},
};
use sqlx::query_as;

use crate::{
    app::state::AppState,
    app::error::AppError,
    domain::models::StoredFile,
    http::handlers::file_page::{build_view_html, content_filename},
};

pub async fn raw_file(
    State(state): State<AppState>,
    Path(requested_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file = find_file(&state, &requested_id).await?;
    serve_file(state, file, false).await
}

pub async fn download_file(
    State(state): State<AppState>,
    Path(requested_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file = find_file(&state, &requested_id).await?;
    serve_file(state, file, true).await
}

pub async fn view_file(
    State(state): State<AppState>,
    Path(requested_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let file = find_file_for_view(&state, &requested_id).await?;

    let html = build_view_html(&state, &file);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
    );

    Ok((StatusCode::OK, headers, Html(html)))
}

async fn find_file(state: &AppState, requested_id: &str) -> Result<StoredFile, AppError> {
    let file_id = requested_id
        .split_once('.')
        .map(|(id, _)| id)
        .unwrap_or(requested_id);

    let file: Option<StoredFile> = query_as::<_, StoredFile>(
        r#"
        SELECT f.id, f.original_name, f.object_key, f.mime_type, f.extension, f.size, f.nsfw, f.uploader, f.uploader_id, u.preferred_hex_colour, f.created_at
        FROM files f
        LEFT JOIN users u ON u.id = f.uploader_id
        WHERE f.id = ?
        LIMIT 1
        "#,
    )
    .bind(file_id)
    .fetch_optional(&state.db)
    .await?;

    file.ok_or(AppError::NotFound("File not found."))
}

async fn find_file_for_view(state: &AppState, requested_id: &str) -> Result<StoredFile, AppError> {
    let (file_id, requested_extension) = requested_id
        .rsplit_once('.')
        .ok_or(AppError::NotFound("File not found."))?;

    if file_id.is_empty() || requested_extension.is_empty() {
        return Err(AppError::NotFound("File not found."));
    }

    let file = find_file(state, file_id).await?;

    if requested_extension != file.extension {
        return Err(AppError::NotFound("File not found."));
    }

    Ok(file)
}

async fn serve_file(
    state: AppState,
    file: StoredFile,
    as_attachment: bool,
) -> Result<impl IntoResponse, AppError> {
    let object = state
        .s3
        .get_object()
        .bucket(&state.config.r2_bucket)
        .key(&file.object_key)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("R2 download failed: {e}")))?;

    let content_type = object
        .content_type()
        .map(str::to_owned)
        .unwrap_or_else(|| file.mime_type.clone());

    let bytes = object
        .body
        .collect()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read object body: {e}")))?
        .into_bytes();

    let disposition_type = if as_attachment {
        "attachment"
    } else {
        "inline"
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&bytes.len().to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            r#"{disposition_type}; filename="{}""#,
            content_filename(&file)
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );

    Ok((StatusCode::OK, headers, Body::from(bytes)))
}

