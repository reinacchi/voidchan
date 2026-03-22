use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

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
    presence::{ActivitySummary, CachedPresence, DiscordUserSummary, spotify_image_url},
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

    let raw_username = truncate(
        non_empty(Some(payload.presence.discord_user.username.as_str()))
            .unwrap_or(payload.presence.discord_user.id.as_str()),
        24,
    );
    let username = if raw_username.starts_with('@') {
        raw_username
    } else {
        format!("@{}", raw_username)
    };

    let display_name = truncate(&preferred_display_name(&payload.presence.discord_user), 24);
    let custom_status_header = custom_status_text(payload).map(|value| truncate(&value, 30));

    let activity_line = truncate(
        &primary_activity_line(payload, query.idle_message.as_deref()),
        28,
    );

    let secondary_lines = secondary_activity_lines(payload, "")
        .into_iter()
        .map(|line| truncate(&line, 30))
        .collect::<Vec<_>>();

    let elapsed_line = elapsed_activity_line(payload)
        .map(|value| truncate(&value, 34))
        .unwrap_or_default();

    let avatar_url = discord_avatar_url(&payload.presence.discord_user);
    let activity_asset_url = widget_activity_asset_url(payload);
    let activity_small_asset_url = widget_activity_small_asset_url(payload);

    let activity_name = truncate(&widget_activity_name(payload), 24);

    let escaped_avatar_url = escape_attr(&avatar_url);
    let escaped_username = escape_html(&username);
    let escaped_display_name = escape_html(&display_name);
    let escaped_activity = escape_html(&activity_line);
    let escaped_desc = escape_attr(&activity_line);
    let escaped_activity_name = escape_html(&activity_name.to_uppercase());
    let custom_status_svg = custom_status_header
        .as_ref()
        .map(|value| {
            format!(
                r#"  <text x="86" y="78" fill="{muted}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="11.5" font-weight="500">{value}</text>"#,
                muted = palette.muted,
                value = escape_html(value),
            )
        })
        .unwrap_or_default();

    let activity_label_y = 126;
    let activity_title_y = 146;
    let secondary_start_y = 162;
    let secondary_gap = 14;
    let elapsed_y = if !elapsed_line.is_empty() {
        secondary_start_y + (secondary_lines.len() as i32 * secondary_gap) + 4
    } else if secondary_lines.is_empty() {
        activity_title_y
    } else {
        secondary_start_y + ((secondary_lines.len() as i32 - 1) * secondary_gap)
    };

    let activity_asset_x = 18;
    let activity_asset_y = 112;
    let activity_asset_min_size = 65;
    let activity_asset_size = if activity_asset_url.is_some() {
        (elapsed_y - activity_asset_y + 4).max(activity_asset_min_size)
    } else {
        activity_asset_min_size
    };
    let activity_asset_radius = 14;

    let small_asset_size = 24;
    let small_asset_x = activity_asset_x + activity_asset_size - small_asset_size + 4;
    let small_asset_y = activity_asset_y + activity_asset_size - small_asset_size + 4;
    let small_asset_radius = 12;

    let activity_asset_markup = activity_asset_url
        .as_deref()
        .map(|url| {
            format!(
                r#"  <rect x="{x}" y="{y}" width="{size}" height="{size}" rx="{radius}" fill="{panel_soft}" stroke="{border}" />
  <image
    href="{asset_url}"
    x="{x}"
    y="{y}"
    width="{size}"
    height="{size}"
    preserveAspectRatio="xMidYMid slice"
    clip-path="url(#activityAssetClip)"
  />
"#,
                x = activity_asset_x,
                y = activity_asset_y,
                size = activity_asset_size,
                radius = activity_asset_radius,
                panel_soft = palette.panel_soft,
                border = palette.border,
                asset_url = escape_attr(url),
            )
        })
        .unwrap_or_default();

    let activity_small_asset_markup = if activity_asset_url.is_some()
        && activity_small_asset_url.is_some()
    {
        activity_small_asset_url
                .as_deref()
                .map(|url| {
                    format!(
                        r#"
  <rect x="{x}" y="{y}" width="{size}" height="{size}" rx="{radius}" fill="{panel}" stroke="{border}" stroke-width="1.5" />
  <image
    href="{asset_url}"
    x="{x}"
    y="{y}"
    width="{size}"
    height="{size}"
    preserveAspectRatio="xMidYMid slice"
    clip-path="url(#activitySmallAssetClip)"
  />
"#,
                        x = small_asset_x,
                        y = small_asset_y,
                        size = small_asset_size,
                        radius = small_asset_radius,
                        panel = palette.panel,
                        border = palette.border,
                        asset_url = escape_attr(url),
                    )
                })
                .unwrap_or_default()
    } else {
        String::new()
    };

    let text_x = if activity_asset_url.is_some() {
        activity_asset_x + activity_asset_size + 16
    } else {
        18
    };

    let secondary_svg = secondary_lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let y = secondary_start_y + (index as i32 * secondary_gap);
            format!(
                r#"  <text x="{x}" y="{y}" fill="{body}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="11.5">{line}</text>"#,
                x = text_x,
                y = y,
                body = palette.body,
                line = escape_html(line),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let elapsed_svg = if !elapsed_line.is_empty() {
        format!(
            r#"  <text x="{x}" y="{y}" fill="{muted}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="11.5">{elapsed}</text>"#,
            x = text_x,
            y = elapsed_y,
            muted = palette.muted,
            elapsed = escape_html(&elapsed_line),
        )
    } else {
        String::new()
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="380" height="210" viewBox="0 0 380 210" role="img" aria-labelledby="title desc">
  <title id="title">Discord presence for {display_name}</title>
  <desc id="desc">{desc}</desc>

  <defs>
    <clipPath id="avatarClip">
      <rect x="18" y="18" width="54" height="54" rx="14" />
    </clipPath>
    <clipPath id="activityAssetClip">
      <rect x="{activity_asset_x}" y="{activity_asset_y}" width="{activity_asset_size}" height="{activity_asset_size}" rx="{activity_asset_radius}" />
    </clipPath>
    <clipPath id="activitySmallAssetClip">
      <rect x="{small_asset_x}" y="{small_asset_y}" width="{small_asset_size}" height="{small_asset_size}" rx="{small_asset_radius}" />
    </clipPath>
  </defs>

  <rect x="1" y="1" width="378" height="208" rx="14" fill="{background}" stroke="{border}" />

  <rect x="18" y="18" width="54" height="54" rx="14" fill="{panel_soft}" stroke="{border}" />
  <image
    href="{avatar_url}"
    x="18"
    y="18"
    width="54"
    height="54"
    preserveAspectRatio="xMidYMid slice"
    clip-path="url(#avatarClip)"
  />

  <circle cx="64" cy="64" r="9" fill="{background}" />
  <circle cx="64" cy="64" r="6" fill="{status_accent}" stroke="{panel}" stroke-width="1.5" />

  <text x="86" y="40" fill="{title_color}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="18" font-weight="700">{display_name}</text>
  <text x="86" y="60" fill="{muted}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="12.5" font-weight="500">{username}</text>
{custom_status_svg}
  <line x1="6" y1="95" x2="374" y2="95" stroke="{border}" stroke-width="1" />

{activity_asset_markup}{activity_small_asset_markup}  <text x="{text_x}" y="{activity_label_y}" fill="{muted}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="10.5" font-weight="700" letter-spacing="0.4px">{activity_name}</text>
  <text x="{text_x}" y="{activity_title_y}" fill="{title_color}" font-family="Inter, Segoe UI, Arial, sans-serif" font-size="14.5" font-weight="700">{activity}</text>
{secondary_svg}
{elapsed_svg}
</svg>"#,
        display_name = escaped_display_name,
        desc = escaped_desc,
        activity_asset_x = activity_asset_x,
        activity_asset_y = activity_asset_y,
        activity_asset_size = activity_asset_size,
        activity_asset_radius = activity_asset_radius,
        small_asset_x = small_asset_x,
        small_asset_y = small_asset_y,
        small_asset_size = small_asset_size,
        small_asset_radius = small_asset_radius,
        background = background,
        border = palette.border,
        panel = palette.panel,
        avatar_url = escaped_avatar_url,
        status_accent = status_accent,
        username = escaped_username,
        title_color = palette.title,
        muted = palette.muted,
        activity_asset_markup = activity_asset_markup,
        activity_small_asset_markup = activity_small_asset_markup,
        custom_status_svg = custom_status_svg,
        text_x = text_x,
        activity_label_y = activity_label_y,
        activity_title_y = activity_title_y,
        activity_name = escaped_activity_name,
        activity = escaped_activity,
        secondary_svg = secondary_svg,
        elapsed_svg = elapsed_svg,
        panel_soft = palette.panel_soft,
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

fn widget_uses_spotify(payload: &PresencePayload) -> bool {
    payload.presence.spotify.is_some()
}

fn widget_spotify_activity(payload: &PresencePayload) -> Option<&ActivitySummary> {
    payload.presence.activities.iter().find(|activity| {
        activity.kind == "listening" || activity.name.eq_ignore_ascii_case("spotify")
    })
}

fn widget_discord_activity(payload: &PresencePayload) -> Option<&ActivitySummary> {
    payload.presence.activities.iter().find(|activity| {
        activity.kind != "custom"
            && activity.kind != "listening"
            && !activity.name.eq_ignore_ascii_case("spotify")
    })
}

fn widget_activity_name(payload: &PresencePayload) -> String {
    if widget_uses_spotify(payload) {
        return "Spotify".to_string();
    }

    widget_discord_activity(payload)
        .and_then(|activity| non_empty(Some(activity.name.as_str())))
        .unwrap_or("Current activity")
        .to_string()
}

fn widget_activity_asset_url(payload: &PresencePayload) -> Option<String> {
    if widget_uses_spotify(payload) {
        return widget_spotify_activity(payload).and_then(activity_asset_image_url);
    }

    widget_discord_activity(payload).and_then(activity_asset_image_url)
}

fn widget_activity_small_asset_url(payload: &PresencePayload) -> Option<String> {
    if widget_uses_spotify(payload) {
        return widget_spotify_activity(payload).and_then(activity_small_asset_image_url);
    }

    widget_discord_activity(payload).and_then(activity_small_asset_image_url)
}

fn primary_activity_line(payload: &PresencePayload, idle_message: Option<&str>) -> String {
    if widget_uses_spotify(payload) {
        if let Some(spotify) = payload.presence.spotify.as_ref() {
            return format!("Listening to {}", spotify.song);
        }
    }

    if let Some(activity) = widget_discord_activity(payload) {
        return format_primary_activity(activity);
    }

    if let Some(custom) = custom_status_text(payload) {
        return custom;
    }

    idle_message
        .unwrap_or("No current Discord activity")
        .to_string()
}

fn secondary_activity_lines(payload: &PresencePayload, status_line: &str) -> Vec<String> {
    if widget_uses_spotify(payload) {
        if let Some(spotify) = payload.presence.spotify.as_ref() {
            let mut lines = Vec::new();

            if !spotify.artist.trim().is_empty() {
                lines.push(format!("By {}", spotify.artist));
            }

            if let Some(album) = spotify
                .album
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            {
                lines.push(format!("on {}", album));
            }

            return lines;
        }
    }

    if let Some(activity) = widget_discord_activity(payload) {
        if let Some(context) = activity_context_lines(activity) {
            return context;
        }

        if let Some(custom) = custom_status_text(payload) {
            return vec![custom];
        }

        return vec![
            pretty_activity_kind(&activity.kind).to_string(),
            activity.name.clone(),
        ];
    }

    if let Some(custom) = custom_status_text(payload) {
        return vec![custom];
    }

    if payload.kv.is_empty() {
        if status_line.trim().is_empty() {
            return Vec::new();
        }

        return vec![status_line.to_string()];
    }

    vec![
        status_line.to_string(),
        format!(
            "{} KV entr{}",
            payload.kv.len(),
            if payload.kv.len() == 1 { "y" } else { "ies" }
        ),
    ]
}

fn activity_asset_image_url(activity: &ActivitySummary) -> Option<String> {
    let assets = activity.assets.as_ref()?;
    let large_image = non_empty(assets.large_image.as_deref())?;

    if let Some(url) = spotify_image_url(large_image) {
        return Some(url);
    }

    if let Some(path) = large_image.strip_prefix("mp:") {
        return Some(format!(
            "https://media.discordapp.net/{}",
            path.trim_start_matches('/')
        ));
    }

    if large_image.starts_with("https://") || large_image.starts_with("http://") {
        return Some(large_image.to_string());
    }

    activity
        .application_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|application_id| {
            format!(
                "https://cdn.discordapp.com/app-assets/{}/{}.png",
                application_id, large_image
            )
        })
}

fn activity_small_asset_image_url(activity: &ActivitySummary) -> Option<String> {
    let assets = activity.assets.as_ref()?;
    let small_image = non_empty(assets.small_image.as_deref())?;

    if let Some(url) = spotify_image_url(small_image) {
        return Some(url);
    }

    if let Some(path) = small_image.strip_prefix("mp:") {
        return Some(format!(
            "https://media.discordapp.net/{}",
            path.trim_start_matches('/')
        ));
    }

    if small_image.starts_with("https://") || small_image.starts_with("http://") {
        return Some(small_image.to_string());
    }

    activity
        .application_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|application_id| {
            format!(
                "https://cdn.discordapp.com/app-assets/{}/{}.png",
                application_id, small_image
            )
        })
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

fn activity_context_lines(activity: &ActivitySummary) -> Option<Vec<String>> {
    match (
        non_empty(activity.details.as_deref()),
        non_empty(activity.state.as_deref()),
    ) {
        (Some(details), Some(state)) => Some(vec![details.to_string(), state.to_string()]),
        (Some(details), None) => Some(vec![details.to_string()]),
        (None, Some(state)) => Some(vec![state.to_string()]),
        (None, None) => None,
    }
}

fn elapsed_activity_line(payload: &PresencePayload) -> Option<String> {
    if widget_uses_spotify(payload) {
        if let Some(spotify) = payload.presence.spotify.as_ref() {
            if let Some(start) = spotify.timestamps.as_ref().and_then(|value| value.start) {
                return Some(format!(
                    "{} elapsed",
                    format_elapsed_from_unix_millis(start as i64)
                ));
            }
        }

        return None;
    }

    if let Some(activity) = widget_discord_activity(payload) {
        if let Some(start) = activity.timestamps.as_ref().and_then(|value| value.start) {
            return Some(format!(
                "{} elapsed",
                format_elapsed_from_unix_millis(start as i64)
            ));
        }
    }

    None
}

fn format_elapsed_from_unix_millis(start_ms: i64) -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(start_ms);

    let elapsed_ms = now_ms.saturating_sub(start_ms);
    let total_seconds = (elapsed_ms / 1000).max(0);

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
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
