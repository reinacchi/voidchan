use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse},
};
use sqlx::query_as;

use crate::{
    app_state::AppState,
    error::AppError,
    models::StoredFile,
    utils::html::{escape_attr, escape_html},
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

    let title = content_filename(&file);
    let description = format!(
        "Uploaded on {}",
        file.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    let raw_url = raw_route_url(&state, &file);
    let download_url = download_route_url(&state, &file);
    let canonical_url = view_route_url(&state, &file);
    let theme_colour = file
        .preferred_hex_colour
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "#7289da".to_string());

    let escaped_title = escape_html(&title);
    let escaped_description = escape_html(&description);
    let escaped_title_attr = escape_attr(&title);
    let escaped_description_attr = escape_attr(&description);
    let escaped_raw_url = escape_attr(&raw_url);
    let escaped_download_url = escape_attr(&download_url);
    let escaped_canonical_url = escape_attr(&canonical_url);
    let escaped_uploader = escape_html(&file.uploader);
    let escaped_mime = escape_html(&file.mime_type);
    let escaped_size = escape_html(&human_file_size(file.size));
    let escaped_theme_colour = escape_attr(&theme_colour);

    let image_or_link = preview_markup(&file.mime_type, &escaped_raw_url, &escaped_title_attr);
    let og_type = open_graph_type(&file.mime_type);
    let twitter_card = twitter_card_type(&file.mime_type);
    let og_image_meta_tag = open_graph_image_meta_tag(&state, &file);
    let twitter_image_meta_tag = twitter_image_meta_tag(&state, &file);
    let video_meta_tags = video_meta_tags(&file, &escaped_raw_url, &escaped_canonical_url);

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <title>{title}</title>

  <meta name="theme-color" content="{theme_colour}" />
  <meta name="description" content="{description_attr}" />
  <meta property="og:type" content="{og_type}" />
  <meta property="og:title" content="{title_attr}" />
  <meta property="og:description" content="{description_attr}" />
  <meta property="og:url" content="{canonical}" />
  <link rel="canonical" href="{canonical}" />
  {og_image_meta_tag}
  {video_meta_tags}
  <meta name="twitter:card" content="{twitter_card}" />
  <meta name="twitter:title" content="{title_attr}" />
  <meta name="twitter:description" content="{description_attr}" />
  {twitter_image_meta_tag}

  <style>
    :root {{
      color-scheme: dark;
      --accent: {theme_colour};
      --accent-soft: color-mix(in srgb, var(--accent) 20%, transparent);
    }}
    * {{
      box-sizing: border-box;
    }}
    body {{
      margin: 0;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background:
        radial-gradient(circle at top, var(--accent-soft), transparent 40%),
        linear-gradient(180deg, #0f1115 0%, #161a22 100%);
      color: #e5e7eb;
    }}
    .wrap {{
      max-width: 1100px;
      margin: 0 auto;
      padding: 32px 20px 48px;
    }}
    .card {{
      background: rgba(24, 24, 27, 0.82);
      border: 1px solid color-mix(in srgb, var(--accent) 25%, rgba(148, 163, 184, 0.14));
      border-radius: 24px;
      padding: 24px;
      box-shadow: 0 20px 50px rgba(0, 0, 0, 0.35);
      backdrop-filter: blur(10px);
    }}
    h1 {{
      margin: 0 0 10px;
      font-size: clamp(1.6rem, 4vw, 2.4rem);
      word-break: break-word;
      color: #f3f4f6;
      letter-spacing: -0.02em;
    }}
    .muted {{
      color: #9ca3af;
    }}
    .meta {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 12px;
      margin: 20px 0 28px;
    }}
    .meta > div {{
      background: rgba(39, 39, 42, 0.92);
      border: 1px solid rgba(82, 82, 91, 0.45);
      border-radius: 16px;
      padding: 14px 16px;
    }}
    .label {{
      display: block;
      color: #a1a1aa;
      font-size: 0.85rem;
      margin-bottom: 6px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }}
    .meta strong {{
      color: #f4f4f5;
    }}
    .actions {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      margin: 0 0 24px;
    }}
    .button {{
      display: inline-flex;
      align-items: center;
      justify-content: center;
      padding: 12px 18px;
      border-radius: 999px;
      border: 1px solid color-mix(in srgb, var(--accent) 35%, rgba(148, 163, 184, 0.22));
      background: rgba(39, 39, 42, 0.92);
      color: #f8fafc;
      font-weight: 700;
      text-decoration: none;
      transition: transform 0.15s ease, border-color 0.15s ease, background 0.15s ease;
    }}
    .button:hover {{
      text-decoration: none;
      transform: translateY(-1px);
      border-color: var(--accent);
      background: color-mix(in srgb, var(--accent) 18%, rgba(39, 39, 42, 0.92));
    }}
    .preview {{
      background: rgba(9, 9, 11, 0.8);
      border-radius: 20px;
      padding: 18px;
      border: 1px solid rgba(82, 82, 91, 0.4);
    }}
    a {{
      color: var(--accent);
      overflow-wrap: anywhere;
      text-decoration: none;
    }}
    a:hover {{
      color: #f8fafc;
      text-decoration: underline;
    }}
  </style>
</head>
<body>
  <div class="wrap">
    <div class="card">
      <h1>{title}</h1>
      <p class="muted">{description}</p>

      <div class="meta">
        <div>
          <span class="label">Uploader</span>
          <strong>{uploader}</strong>
        </div>
        <div>
          <span class="label">Type</span>
          <strong>{mime}</strong>
        </div>
        <div>
          <span class="label">Size</span>
          <strong>{size}</strong>
        </div>
        <div>
          <span class="label">Raw URL</span>
          <a href="{raw}">{raw}</a>
        </div>
      </div>

      <div class="actions">
        <a class="button" href="{download}" download>Download file</a>
        <a class="button" href="{raw}" target="_blank" rel="noreferrer">Open raw file</a>
      </div>

      <div class="preview">
        {image_or_link}
      </div>
    </div>
  </div>
</body>
</html>"#,
        title = escaped_title,
        title_attr = escaped_title_attr,
        description = escaped_description,
        description_attr = escaped_description_attr,
        canonical = escaped_canonical_url,
        raw = escaped_raw_url,
        og_image_meta_tag = og_image_meta_tag,
        twitter_image_meta_tag = twitter_image_meta_tag,
        video_meta_tags = video_meta_tags,
        download = escaped_download_url,
        uploader = escaped_uploader,
        mime = escaped_mime,
        size = escaped_size,
        image_or_link = image_or_link,
        theme_colour = escaped_theme_colour,
        og_type = og_type,
        twitter_card = twitter_card,
    );

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

fn raw_route_url(state: &AppState, file: &StoredFile) -> String {
    format!(
        "{}/u/{}",
        state.config.base_url.trim_end_matches('/'),
        content_filename(file)
    )
}

fn download_route_url(state: &AppState, file: &StoredFile) -> String {
    format!(
        "{}/download/{}",
        state.config.base_url.trim_end_matches('/'),
        content_filename(file)
    )
}

fn view_route_url(state: &AppState, file: &StoredFile) -> String {
    format!(
        "{}/v/{}",
        state.config.base_url.trim_end_matches('/'),
        content_filename(file)
    )
}

fn content_filename(file: &StoredFile) -> String {
    format!("{}.{}", file.id, file.extension)
}

fn human_file_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if size == 0 {
        return "0 B".to_string();
    }

    let mut value = size as f64;
    let mut unit_index = 0usize;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{value:.2} {}", UNITS[unit_index])
    }
}


fn preview_markup(mime_type: &str, raw_url: &str, title_attr: &str) -> String {
    if mime_type.starts_with("image/") {
        return format!(
            r#"<img src="{raw}" alt="{alt}" style="max-width:min(100%, 960px); max-height:70vh; border-radius:16px; display:block; margin:0 auto;" />"#,
            raw = raw_url,
            alt = title_attr
        );
    }

    if mime_type.starts_with("video/") {
        return format!(
            r#"<video controls playsinline preload="metadata" style="width:min(100%, 960px); max-height:70vh; border-radius:16px; display:block; margin:0 auto; background:#000;">
  <source src="{raw}" type="{mime}">
  Your browser does not support the video tag.
</video>"#,
            raw = raw_url,
            mime = escape_attr(mime_type)
        );
    }

    if mime_type.starts_with("audio/") {
        return format!(
            r#"<audio controls preload="metadata" style="width:min(100%, 720px); display:block; margin:0 auto;">
  <source src="{raw}" type="{mime}">
  Your browser does not support the audio element.
</audio>"#,
            raw = raw_url,
            mime = escape_attr(mime_type)
        );
    }

    format!(
        r#"<p style="text-align:center; margin:2rem 0;"><a href="{raw}" style="color:var(--accent); font-weight:600; text-decoration:none;">Open raw file</a></p>"#,
        raw = raw_url
    )
}

fn open_graph_image_meta_tag(state: &AppState, file: &StoredFile) -> String {
    if !file.mime_type.starts_with("image/") {
        return String::new();
    }

    format!(
        r#"<meta property="og:image" content="{}" />"#,
        escape_attr(&raw_route_url(state, file))
    )
}

fn twitter_image_meta_tag(state: &AppState, file: &StoredFile) -> String {
    if !file.mime_type.starts_with("image/") {
        return String::new();
    }

    format!(
        r#"<meta name="twitter:image" content="{}" />"#,
        escape_attr(&raw_route_url(state, file))
    )
}

fn video_meta_tags(file: &StoredFile, raw_url: &str, canonical_url: &str) -> String {
    if !file.mime_type.starts_with("video/") {
        return String::new();
    }

    let mime = escape_attr(&file.mime_type);

    format!(
        r#"<meta property="og:video" content="{raw}" />
  <meta property="og:video:url" content="{raw}" />
  <meta property="og:video:secure_url" content="{raw}" />
  <meta property="og:video:type" content="{mime}" />
  <meta property="og:video:width" content="1280" />
  <meta property="og:video:height" content="720" />
  <meta name="twitter:url" content="{canonical}" />
  <meta name="twitter:player" content="{canonical}" />
  <meta name="twitter:player:width" content="1280" />
  <meta name="twitter:player:height" content="720" />
  <meta name="twitter:player:stream" content="{raw}" />
  <meta name="twitter:player:stream:content_type" content="{mime}" />"#,
        raw = raw_url,
        mime = mime,
        canonical = canonical_url,
    )
}

fn open_graph_type(mime_type: &str) -> String {
    if mime_type.starts_with("video/") {
        mime_type.to_string()
    } else {
        "website".to_string()
    }
}

fn twitter_card_type(mime_type: &str) -> &'static str {
    if mime_type.starts_with("video/") {
        "player"
    } else {
        "summary_large_image"
    }
}
