use aws_sdk_s3::primitives::ByteStream;
use axum::{
    Json,
    extract::{Multipart, State},
    http::HeaderMap,
};
use chrono::Utc;
use sqlx::query_as;

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

    let mut uploaded = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("Invalid multipart form data."))?
    {
        let original_name = field.file_name().map(|s| s.to_string());
        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = field
            .bytes()
            .await
            .map_err(|_| AppError::BadRequest("Failed to read uploaded file."))?;

        uploaded = Some((original_name, content_type, data.to_vec()));
        break;
    }

    let (original_name, mime_type, bytes) =
        uploaded.ok_or(AppError::BadRequest("No file uploaded."))?;

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

    state
        .s3
        .put_object()
        .bucket(&state.config.r2_bucket)
        .key(&object_key)
        .content_type(mime_type.clone())
        .body(ByteStream::from(bytes.clone()))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("R2 upload failed: {e}")))?;

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
    .bind(bytes.len() as u64)
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
