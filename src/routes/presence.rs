use std::collections::{HashMap, HashSet};

use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

use crate::{
    app_state::AppState,
    error::AppError,
    models::User,
    presence::{ActivitySummary, CachedPresence, DiscordUserSummary},
    utils::{
        html::{escape_attr, escape_html},
        ids::is_valid_hex_colour,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct PresenceEnvelope {
    pub success: bool,
    pub data: PresencePayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresencePayload {
    #[serde(flatten)]
    pub presence: CachedPresence,
    pub kv: HashMap<String, String>,
}

#[derive(Debug, Clone, FromRow)]
struct KvRow {
    kv_key: String,
    kv_value: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct WidgetQuery {
    pub theme: Option<String>,
    pub accent: Option<String>,
    pub idle_message: Option<String>,
}

pub async fn get_presence(
    State(state): State<AppState>,
    Path(discord_user_id): Path<String>,
) -> Result<(StatusCode, HeaderMap, Json<PresenceEnvelope>), AppError> {
    let payload = load_presence_payload(&state, &discord_user_id).await?;

    Ok(json_response(
        StatusCode::OK,
        PresenceEnvelope {
            success: true,
            data: payload,
        },
    ))
}

pub async fn put_presence_kv(
    State(state): State<AppState>,
    Path((discord_user_id, key)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, HeaderMap, Json<PresenceEnvelope>), AppError> {
    validate_kv_key(&key)?;
    let value = String::from_utf8_lossy(&body).to_string();
    validate_kv_value(&value)?;

    let user = authorised_kv_user(&state, &headers, &discord_user_id).await?;
    ensure_kv_capacity(&state, user.id, &[key.clone()]).await?;

    sqlx::query(
        r#"
        INSERT INTO user_presence_kv (user_id, kv_key, kv_value)
        VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE kv_value = VALUES(kv_value), updated_at = UTC_TIMESTAMP()
        "#,
    )
    .bind(user.id)
    .bind(&key)
    .bind(&value)
    .execute(&state.db)
    .await?;

    let payload = load_presence_payload(&state, &discord_user_id).await?;

    Ok(json_response(
        StatusCode::OK,
        PresenceEnvelope {
            success: true,
            data: payload,
        },
    ))
}

pub async fn patch_presence_kv(
    State(state): State<AppState>,
    Path(discord_user_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<HashMap<String, Value>>,
) -> Result<(StatusCode, HeaderMap, Json<PresenceEnvelope>), AppError> {
    let user = authorised_kv_user(&state, &headers, &discord_user_id).await?;

    let mut items = Vec::with_capacity(body.len());
    for (key, value) in body {
        validate_kv_key(&key)?;
        let value = scalar_json_to_string(value)?;
        validate_kv_value(&value)?;
        items.push((key, value));
    }

    let keys = items.iter().map(|(key, _)| key.clone()).collect::<Vec<_>>();
    ensure_kv_capacity(&state, user.id, &keys).await?;

    for (key, value) in items {
        sqlx::query(
            r#"
            INSERT INTO user_presence_kv (user_id, kv_key, kv_value)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE kv_value = VALUES(kv_value), updated_at = UTC_TIMESTAMP()
            "#,
        )
        .bind(user.id)
        .bind(&key)
        .bind(&value)
        .execute(&state.db)
        .await?;
    }

    let payload = load_presence_payload(&state, &discord_user_id).await?;

    Ok(json_response(
        StatusCode::OK,
        PresenceEnvelope {
            success: true,
            data: payload,
        },
    ))
}

pub async fn delete_presence_kv(
    State(state): State<AppState>,
    Path((discord_user_id, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Json<PresenceEnvelope>), AppError> {
    validate_kv_key(&key)?;
    let user = authorised_kv_user(&state, &headers, &discord_user_id).await?;

    sqlx::query("DELETE FROM user_presence_kv WHERE user_id = ? AND kv_key = ?")
        .bind(user.id)
        .bind(&key)
        .execute(&state.db)
        .await?;

    let payload = load_presence_payload(&state, &discord_user_id).await?;

    Ok(json_response(
        StatusCode::OK,
        PresenceEnvelope {
            success: true,
            data: payload,
        },
    ))
}

pub async fn presence_widget_svg(
    State(state): State<AppState>,
    Path(discord_user_id): Path<String>,
    Query(query): Query<WidgetQuery>,
) -> Result<(StatusCode, HeaderMap, String), AppError> {
    let payload = load_presence_payload(&state, &discord_user_id).await?;
    let svg = render_widget_svg(&payload, &query);

    Ok(svg_response(svg))
}

async fn load_presence_payload(
    state: &AppState,
    discord_user_id: &str,
) -> Result<PresencePayload, AppError> {
    let registered_user = registered_user_by_discord_id(state, discord_user_id).await?;
    let kv = if let Some(user) = registered_user.as_ref() {
        load_kv_for_user(state, user.id).await?
    } else {
        HashMap::new()
    };

    let mut presence = match (
        state.presence.get(discord_user_id).await,
        registered_user.as_ref(),
    ) {
        (Some(presence), _) => presence,
        (None, Some(user)) => {
            CachedPresence::offline_for_registered_user(discord_user_id, &user.username)
        }
        (None, None) => return Err(AppError::NotFound("Discord presence not found.")),
    };

    if presence.discord_user.username.trim().is_empty()
        || presence.discord_user.username == presence.discord_user.id
    {
        if let Some(user) = registered_user.as_ref() {
            presence.discord_user.username = user.username.clone();
        }
    }

    if (presence.discord_user.display_name.trim().is_empty()
        || presence.discord_user.display_name == presence.discord_user.id)
        && !presence.discord_user.username.trim().is_empty()
        && presence.discord_user.username != presence.discord_user.id
    {
        presence.discord_user.display_name = presence.discord_user.username.clone();
    }

    Ok(PresencePayload { presence, kv })
}

async fn registered_user_by_discord_id(
    state: &AppState,
    discord_user_id: &str,
) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, api_token, discord_user_id, preferred_url_mode, preferred_hex_colour, is_blacklisted, created_at
        FROM users
        WHERE discord_user_id = ?
        LIMIT 1
        "#,
    )
    .bind(discord_user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(Into::into)
}

async fn load_kv_for_user(
    state: &AppState,
    user_id: u64,
) -> Result<HashMap<String, String>, AppError> {
    let rows = sqlx::query_as::<_, KvRow>(
        r#"
        SELECT kv_key, kv_value
        FROM user_presence_kv
        WHERE user_id = ?
        ORDER BY kv_key ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| (row.kv_key, row.kv_value))
        .collect())
}

async fn authorised_kv_user(
    state: &AppState,
    headers: &HeaderMap,
    discord_user_id: &str,
) -> Result<User, AppError> {
    let auth = headers
        .get("Authorisation")
        .or_else(|| headers.get("Authorization"))
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::Unauthorized(
            "You are not authorised to use this endpoint.",
        ))?;

    let user: Option<User> = sqlx::query_as::<_, User>(
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

    if user.discord_user_id.as_deref() != Some(discord_user_id) {
        return Err(AppError::Unauthorized(
            "This API token does not belong to that Discord user.",
        ));
    }

    Ok(user)
}

async fn ensure_kv_capacity(
    state: &AppState,
    user_id: u64,
    candidate_keys: &[String],
) -> Result<(), AppError> {
    let existing_keys =
        sqlx::query_as::<_, (String,)>("SELECT kv_key FROM user_presence_kv WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(&state.db)
            .await?;

    let mut all_keys = existing_keys
        .into_iter()
        .map(|(key,)| key)
        .collect::<HashSet<_>>();

    for key in candidate_keys {
        all_keys.insert(key.clone());
    }

    if all_keys.len() > 512 {
        return Err(AppError::BadRequest(
            "KV stores are limited to 512 keys per user.",
        ));
    }

    Ok(())
}

fn validate_kv_key(key: &str) -> Result<(), AppError> {
    if key.is_empty()
        || key.len() > 255
        || !key
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return Err(AppError::BadRequest(
            "KV keys must be alphanumeric and at most 255 characters long.",
        ));
    }

    Ok(())
}

fn validate_kv_value(value: &str) -> Result<(), AppError> {
    if value.chars().count() > 30_000 {
        return Err(AppError::BadRequest(
            "KV values may not exceed 30000 characters.",
        ));
    }

    Ok(())
}

fn scalar_json_to_string(value: Value) -> Result<String, AppError> {
    match value {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Ok("null".to_string()),
        Value::Array(_) | Value::Object(_) => Err(AppError::BadRequest(
            "KV patch values must be strings, numbers, booleans, or null.",
        )),
    }
}

fn json_response<T: Serialize>(status: StatusCode, value: T) -> (StatusCode, HeaderMap, Json<T>) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));

    (status, headers, Json(value))
}

fn svg_response(svg: String) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("image/svg+xml; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));

    (StatusCode::OK, headers, svg)
}

fn render_widget_svg(payload: &PresencePayload, query: &WidgetQuery) -> String {
    let palette = palette(query.theme.as_deref());
    let background = query
        .accent
        .as_deref()
        .filter(|value| is_valid_hex_colour(value))
        .unwrap_or(palette.bg);

    let status_accent = status_colour(&payload.presence.discord_status);

    let username = truncate(
        non_empty(Some(payload.presence.discord_user.username.as_str()))
            .unwrap_or(payload.presence.discord_user.id.as_str()),
        28,
    );
    let display_name = truncate(&preferred_display_name(&payload.presence.discord_user), 28);

    let activity_line = truncate(
        &primary_activity_line(payload, query.idle_message.as_deref()),
        48,
    );

    let footer_line = truncate(&secondary_activity_line(payload, ""), 56);

    let avatar_url = discord_avatar_url(&payload.presence.discord_user);
    let escaped_avatar_url = escape_attr(&avatar_url);
    let escaped_username = escape_html(&username);
    let escaped_display_name = escape_html(&display_name);
    let escaped_activity = escape_html(&activity_line);
    let escaped_footer = escape_html(&footer_line);
    let escaped_desc = escape_attr(&activity_line);

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="420" height="160" viewBox="0 0 420 160" role="img" aria-labelledby="title desc">
  <title id="title">Discord presence for {display_name}</title>
  <desc id="desc">{desc}</desc>

  <defs>
    <clipPath id="avatarClip">
      <rect x="20" y="20" width="64" height="64" rx="12" />
    </clipPath>
  </defs>

  <rect x="1" y="1" width="418" height="158" rx="12" fill="{background}" stroke="{border}" />

  <rect x="20" y="20" width="64" height="64" rx="12" fill="{panel_soft}" stroke="{border}" />
  <image
    href="{avatar_url}"
    x="20"
    y="20"
    width="64"
    height="64"
    preserveAspectRatio="xMidYMid slice"
    clip-path="url(#avatarClip)"
  />

  <circle cx="78" cy="78" r="9" fill="{background}" />
  <circle cx="78" cy="78" r="6" fill="{status_accent}" stroke="{panel}" stroke-width="1.5" />

  <text x="100" y="36" fill="{muted}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="12" font-weight="600">@{username}</text>
  <text x="100" y="58" fill="{title_color}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="18" font-weight="700">{display_name}</text>

  <!-- pushed HR down -->
  <line x1="20" y1="110" x2="400" y2="110" stroke="{border}" stroke-width="1" />

  <!-- more breathing room -->
  <text x="20" y="132" fill="{body}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="14">{activity}</text>
  <text x="20" y="148" fill="{muted}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="11">{footer}</text>
</svg>"#,
        display_name = escaped_display_name,
        desc = escaped_desc,
        background = background,
        border = palette.border,
        panel = palette.panel,
        panel_soft = palette.panel_soft,
        avatar_url = escaped_avatar_url,
        status_accent = status_accent,
        username = escaped_username,
        title_color = palette.title,
        muted = palette.muted,
        body = palette.body,
        activity = escaped_activity,
        footer = escaped_footer,
    )
}

fn discord_avatar_url(user: &DiscordUserSummary) -> String {
    if let Some(avatar) = non_empty(user.avatar.as_deref()) {
        let ext = if avatar.starts_with("a_") {
            "gif"
        } else {
            "png"
        };
        return format!(
            "https://cdn.discordapp.com/avatars/{}/{}.{}?size=128",
            user.id, avatar, ext
        );
    }

    let discriminator_mod = user.id.parse::<u64>().map(|id| (id >> 22) % 6).unwrap_or(0);

    format!(
        "https://cdn.discordapp.com/embed/avatars/{}.png",
        discriminator_mod
    )
}

fn primary_activity_line(payload: &PresencePayload, idle_message: Option<&str>) -> String {
    if let Some(spotify) = payload.presence.spotify.as_ref() {
        return format!("Listening to {}", spotify.song);
    }

    if let Some(activity) = rich_activity(payload) {
        return format_primary_activity(activity);
    }

    if let Some(custom) = custom_status_text(payload) {
        return custom;
    }

    idle_message
        .unwrap_or("No current Discord activity")
        .to_string()
}

fn secondary_activity_line(payload: &PresencePayload, status_line: &str) -> String {
    if let Some(spotify) = payload.presence.spotify.as_ref() {
        return match spotify.album.as_deref() {
            Some(album) if !album.trim().is_empty() => {
                format!("{} • {}", spotify.artist, album)
            }
            _ => spotify.artist.clone(),
        };
    }

    if let Some(activity) = rich_activity(payload) {
        if let Some(context) = activity_context_line(activity) {
            return context;
        }

        if let Some(custom) = custom_status_text(payload) {
            return custom;
        }

        return format!(
            "{} • {}",
            pretty_activity_kind(&activity.kind),
            activity.name
        );
    }

    if let Some(custom) = custom_status_text(payload) {
        return custom;
    }

    if payload.kv.is_empty() {
        status_line.to_string()
    } else {
        format!(
            "{} • {} KV entr{}",
            status_line,
            payload.kv.len(),
            if payload.kv.len() == 1 { "y" } else { "ies" }
        )
    }
}

fn rich_activity(payload: &PresencePayload) -> Option<&ActivitySummary> {
    payload
        .presence
        .activities
        .iter()
        .find(|activity| activity.kind != "custom")
}

fn custom_status_text(payload: &PresencePayload) -> Option<String> {
    let custom = payload
        .presence
        .activities
        .iter()
        .find(|activity| activity.kind == "custom")?;

    let emoji = custom
        .emoji
        .as_ref()
        .and_then(|value| non_empty(Some(value.name.as_str())));
    let state = non_empty(custom.state.as_deref());

    match (emoji, state) {
        (Some(emoji), Some(state)) => Some(format!("{} {}", emoji, state)),
        (Some(emoji), None) => Some(emoji.to_string()),
        (None, Some(state)) => Some(state.to_string()),
        (None, None) => None,
    }
}

fn format_primary_activity(activity: &ActivitySummary) -> String {
    if let Some(name) = non_empty(Some(activity.name.as_str())) {
        return match activity.kind.as_str() {
            "playing" => format!("Playing {}", name),
            "streaming" => format!("Streaming {}", name),
            "listening" => format!("Listening to {}", name),
            "watching" => format!("Watching {}", name),
            "competing" => format!("Competing in {}", name),
            _ => name.to_string(),
        };
    }

    non_empty(activity.details.as_deref())
        .or_else(|| non_empty(activity.state.as_deref()))
        .map(|value| value.to_string())
        .unwrap_or_else(|| pretty_activity_kind(&activity.kind).to_string())
}

fn activity_context_line(activity: &ActivitySummary) -> Option<String> {
    match (
        non_empty(activity.details.as_deref()),
        non_empty(activity.state.as_deref()),
    ) {
        (Some(details), Some(state)) => Some(format!("{} • {}", details, state)),
        (Some(details), None) => Some(details.to_string()),
        (None, Some(state)) => Some(state.to_string()),
        (None, None) => None,
    }
}

fn preferred_display_name(user: &DiscordUserSummary) -> String {
    non_empty(Some(user.display_name.as_str()))
        .filter(|value| *value != user.id.as_str())
        .or_else(|| {
            non_empty(Some(user.username.as_str())).filter(|value| *value != user.id.as_str())
        })
        .unwrap_or(user.id.as_str())
        .to_string()
}

fn non_empty<'a>(value: Option<&'a str>) -> Option<&'a str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn pretty_activity_kind(kind: &str) -> &'static str {
    match kind {
        "playing" => "Playing",
        "streaming" => "Streaming",
        "listening" => "Listening",
        "watching" => "Watching",
        "competing" => "Competing",
        "custom" => "Custom status",
        _ => "Activity",
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }

    let truncated = value
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    format!("{}…", truncated)
}

fn status_colour(status: &str) -> &'static str {
    match status {
        "online" => "#22c55e",
        "idle" => "#f59e0b",
        "dnd" => "#ef4444",
        "offline" => "#64748b",
        _ => "#7c3aed",
    }
}

struct Palette {
    bg: &'static str,
    panel: &'static str,
    panel_soft: &'static str,
    border: &'static str,
    title: &'static str,
    body: &'static str,
    muted: &'static str,
}

fn palette(theme: Option<&str>) -> Palette {
    match theme.unwrap_or("dark").to_ascii_lowercase().as_str() {
        "light" => Palette {
            bg: "#eef2ff",
            panel: "#ffffff",
            panel_soft: "#f8fafc",
            border: "#dbe4f0",
            title: "#0f172a",
            body: "#1e293b",
            muted: "#475569",
        },
        _ => Palette {
            bg: "#0f1117",
            panel: "#181c25",
            panel_soft: "#10141c",
            border: "#2b3444",
            title: "#f8fafc",
            body: "#e2e8f0",
            muted: "#94a3b8",
        },
    }
}
