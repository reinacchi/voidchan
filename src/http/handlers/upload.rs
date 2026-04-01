use std::{env, path::PathBuf};

use aws_sdk_s3::primitives::ByteStream;
use axum::{
    Json,
    extract::{Multipart, State},
    http::HeaderMap,
};
use chrono::Utc;
use sqlx::query_as;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::warn;
use uuid::Uuid;

use crate::{
    app::error::AppError,
    app::state::AppState,
    domain::models::{UploadResponse, User},
    utils::ids::generate_id,
    utils::mime_ext::extension_from_mime,
};

pub async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    let auth = headers
        .get("Authorisation")
        .or_else(|| headers.get("Authorization"))
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(AppError::Unauthorized(
            "You are not authorised to use this endpoint.",
        ))?;

    let user: Option<User> = query_as::<_, User>(
        r#"
        SELECT id, username, api_token, discord_user_id, preferred_url_mode, preferred_hex_colour, is_blacklisted, created_at
        FROM users
        WHERE api_token = ?
        LIMIT 1
        "#,
    )
    .bind(auth)
    .fetch_optional(&state.db)
    .await?;

    let user = user.ok_or(AppError::Unauthorized(
        "You are not authorised to use this endpoint.",
    ))?;

    if user.is_blacklisted {
        return Err(AppError::Unauthorized(
            "Your account is blacklisted from using the API.",
        ));
    }

    let max_upload_bytes = state.config.max_upload_size_mb * 1024 * 1024;
    let mut uploaded = None;

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(leak_string(format!("Invalid multipart form data: {e}")))
    })? {
        let field_name = field.name().map(str::to_string);
        let original_name = field.file_name().map(|s| s.to_string());

        if original_name.is_none() {
            continue;
        }

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let temp_path = temp_upload_path();
        let mut temp_file = File::create(&temp_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create temp upload file: {e}")))?;

        let mut size = 0usize;

        loop {
            let chunk = field.chunk().await.map_err(|e| {
                AppError::BadRequest(leak_string(format!("Failed to read uploaded file: {e}")))
            })?;

            let Some(chunk) = chunk else {
                break;
            };

            size += chunk.len();
            if size > max_upload_bytes {
                let _ = fs::remove_file(&temp_path).await;
                return Err(AppError::PayloadTooLarge(leak_string(format!(
                    "Uploaded file exceeds the {} MB limit.",
                    state.config.max_upload_size_mb
                ))));
            }

            temp_file.write_all(&chunk).await.map_err(|e| {
                AppError::Internal(format!("Failed to write temp upload file: {e}"))
            })?;
        }

        temp_file
            .flush()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to flush temp upload file: {e}")))?;

        uploaded = Some((
            field_name,
            original_name,
            content_type,
            size as u64,
            temp_path,
        ));
        break;
    }

    let (field_name, original_name, mime_type, size, temp_path) =
        uploaded.ok_or(AppError::BadRequest("No file uploaded."))?;

    if field_name.as_deref() != Some("file") {
        warn!(field_name = ?field_name, "Received upload with a non-standard multipart field name; accepting it.");
    }

    let mut id = generate_id(5);
    for _ in 0..5 {
        let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM files WHERE id = ? LIMIT 1")
            .bind(&id)
            .fetch_optional(&state.db)
            .await?;

        if exists.is_none() {
            break;
        }

        id = generate_id(5);
    }

    let extension = extension_from_mime(&mime_type, original_name.as_deref());
    let object_key = format!("uploads/{}.{}", id, extension);

    let body = ByteStream::from_path(&temp_path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to prepare upload stream: {e}")))?;

    let upload_result = state
        .s3
        .put_object()
        .bucket(&state.config.r2_bucket)
        .key(&object_key)
        .content_type(mime_type.clone())
        .body(body)
        .send()
        .await;

    let _ = fs::remove_file(&temp_path).await;

    upload_result.map_err(|e| AppError::Internal(format!("R2 upload failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO files
            (id, original_name, object_key, mime_type, extension, size, nsfw, uploader, uploader_id, created_at)
        VALUES
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(original_name)
    .bind(&object_key)
    .bind(&mime_type)
    .bind(&extension)
    .bind(size)
    .bind(false)
    .bind(&user.username)
    .bind(user.id)
    .bind(Utc::now().naive_utc())
    .execute(&state.db)
    .await?;

    let mode = if user.preferred_url_mode == "u" {
        "u"
    } else {
        "v"
    };
    let url = format!(
        "{}/{}/{}.{}",
        state.config.base_url.trim_end_matches('/'),
        mode,
        id,
        extension
    );

    Ok(Json(UploadResponse { code: 200, url }))
}

fn temp_upload_path() -> PathBuf {
    env::temp_dir().join(format!("voidchan-upload-{}", Uuid::new_v4()))
}

fn leak_string(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
