use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use serde::Serialize;
use serenity::all::{
    Activity, ActivityAssets, ActivityEmoji, ActivityTimestamps, ActivityType, Guild, Member,
    OnlineStatus, Presence, PresenceUser, User,
};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct PresenceService {
    cache: Arc<RwLock<HashMap<String, CachedPresence>>>,
}

impl PresenceService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn sync_guild(&self, guild: &Guild) {
        let mut cache = self.cache.write().await;

        for member in guild.members.values() {
            cache
                .entry(member.user.id.get().to_string())
                .and_modify(|existing| {
                    existing.discord_user = DiscordUserSummary::from_user(&member.user);
                })
                .or_insert_with(|| CachedPresence::offline_for_user(&member.user));
        }

        for presence in guild.presences.values() {
            let key = presence.user.id.get().to_string();
            let fallback_user = cache
                .get(&key)
                .map(|existing| existing.discord_user.clone());

            cache.insert(
                key,
                CachedPresence::from_presence_with_fallback(presence, fallback_user.as_ref()),
            );
        }
    }

    pub async fn upsert_member(&self, member: &Member) {
        self.upsert_user(&member.user).await;
    }

    pub async fn upsert_user(&self, user: &User) {
        let mut cache = self.cache.write().await;
        let key = user.id.get().to_string();

        if let Some(existing) = cache.get_mut(&key) {
            existing.discord_user = DiscordUserSummary::from_user(user);
            return;
        }

        cache.insert(key, CachedPresence::offline_for_user(user));
    }

    pub async fn remove_user(&self, user_id: u64) {
        let mut cache = self.cache.write().await;
        let key = user_id.to_string();

        if let Some(existing) = cache.get_mut(&key) {
            existing.active_on_discord_web = false;
            existing.active_on_discord_desktop = false;
            existing.active_on_discord_mobile = false;
            existing.discord_status = "offline".to_string();
            existing.activities.clear();
            existing.listening_to_spotify = false;
            existing.spotify = None;
            existing.last_updated = Utc::now().timestamp_millis();
            return;
        }

        cache.remove(&key);
    }

    pub async fn update_presence(&self, presence: &Presence) {
        let mut cache = self.cache.write().await;
        let key = presence.user.id.get().to_string();
        let existing = cache.get(&key).cloned();
        let fallback_user = existing.as_ref().map(|value| value.discord_user.clone());

        cache.insert(
            key,
            CachedPresence::from_presence_with_existing(
                presence,
                fallback_user.as_ref(),
                existing.as_ref(),
            ),
        );
    }

    pub async fn get(&self, user_id: &str) -> Option<CachedPresence> {
        self.cache.read().await.get(user_id).cloned()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CachedPresence {
    pub active_on_discord_web: bool,
    pub active_on_discord_desktop: bool,
    pub active_on_discord_mobile: bool,
    pub discord_user: DiscordUserSummary,
    pub discord_status: String,
    pub activities: Vec<ActivitySummary>,
    pub listening_to_spotify: bool,
    pub spotify: Option<SpotifySummary>,
    pub last_updated: i64,
}

#[allow(dead_code)]
impl CachedPresence {
    pub fn from_presence(presence: &Presence) -> Self {
        Self::from_presence_with_existing(presence, None, None)
    }

    pub fn from_presence_with_fallback(
        presence: &Presence,
        fallback_user: Option<&DiscordUserSummary>,
    ) -> Self {
        Self::from_presence_with_existing(presence, fallback_user, None)
    }

    pub fn from_presence_with_existing(
        presence: &Presence,
        fallback_user: Option<&DiscordUserSummary>,
        existing: Option<&CachedPresence>,
    ) -> Self {
        let active_on_discord_web = presence
            .client_status
            .as_ref()
            .map(|status| status.web.is_some())
            .or_else(|| existing.map(|value| value.active_on_discord_web))
            .unwrap_or(false);
        let active_on_discord_desktop = presence
            .client_status
            .as_ref()
            .map(|status| status.desktop.is_some())
            .or_else(|| existing.map(|value| value.active_on_discord_desktop))
            .unwrap_or(false);
        let active_on_discord_mobile = presence
            .client_status
            .as_ref()
            .map(|status| status.mobile.is_some())
            .or_else(|| existing.map(|value| value.active_on_discord_mobile))
            .unwrap_or(false);

        let is_offline = matches!(
            presence.status,
            OnlineStatus::Offline | OnlineStatus::Invisible
        );
        let activities = if is_offline {
            Vec::new()
        } else {
            presence
                .activities
                .iter()
                .map(ActivitySummary::from_activity)
                .collect::<Vec<_>>()
        };
        let spotify = if is_offline {
            None
        } else {
            activities
                .iter()
                .find(|activity| is_spotify_summary(activity))
                .map(SpotifySummary::from_summary)
        };

        Self {
            active_on_discord_web,
            active_on_discord_desktop,
            active_on_discord_mobile,
            discord_user: DiscordUserSummary::from_presence_user_with_fallback(
                &presence.user,
                fallback_user,
            ),
            discord_status: online_status_to_string(&presence.status),
            listening_to_spotify: spotify.is_some(),
            spotify,
            activities,
            last_updated: Utc::now().timestamp_millis(),
        }
    }

    pub fn offline_for_user(user: &User) -> Self {
        Self {
            active_on_discord_web: false,
            active_on_discord_desktop: false,
            active_on_discord_mobile: false,
            discord_user: DiscordUserSummary::from_user(user),
            discord_status: "offline".to_string(),
            activities: Vec::new(),
            listening_to_spotify: false,
            spotify: None,
            last_updated: Utc::now().timestamp_millis(),
        }
    }

    pub fn offline_for_registered_user(discord_user_id: &str, username: &str) -> Self {
        Self {
            active_on_discord_web: false,
            active_on_discord_desktop: false,
            active_on_discord_mobile: false,
            discord_user: DiscordUserSummary::offline(discord_user_id, username),
            discord_status: "offline".to_string(),
            activities: Vec::new(),
            listening_to_spotify: false,
            spotify: None,
            last_updated: Utc::now().timestamp_millis(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DiscordUserSummary {
    pub id: String,
    pub username: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub avatar_url: String,
    pub bot: bool,
}

#[allow(dead_code)]
impl DiscordUserSummary {
    pub fn from_user(user: &User) -> Self {
        Self {
            id: user.id.get().to_string(),
            username: user.name.clone(),
            display_name: user
                .global_name
                .clone()
                .unwrap_or_else(|| user.name.clone()),
            global_name: user.global_name.clone(),
            discriminator: user
                .discriminator
                .as_ref()
                .map(|value| format!("{:04}", value.get())),
            avatar: user.avatar.as_ref().map(ToString::to_string),
            avatar_url: user
                .avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
            bot: user.bot,
        }
    }

    pub fn from_presence_user(user: &PresenceUser) -> Self {
        Self::from_presence_user_with_fallback(user, None)
    }

    pub fn from_presence_user_with_fallback(
        user: &PresenceUser,
        fallback: Option<&DiscordUserSummary>,
    ) -> Self {
        if let Some(full_user) = user.to_user() {
            return Self::from_user(&full_user);
        }

        let id = user.id.get().to_string();
        let username = user
            .name
            .clone()
            .or_else(|| {
                fallback.and_then(|value| non_empty_non_id(Some(value.username.as_str()), &id))
            })
            .unwrap_or_else(|| id.clone());
        let global_name = user
            .global_name
            .clone()
            .or_else(|| fallback.and_then(|value| non_empty_string(value.global_name.as_deref())));
        let display_name = global_name
            .clone()
            .or_else(|| {
                fallback.and_then(|value| non_empty_non_id(Some(value.display_name.as_str()), &id))
            })
            .unwrap_or_else(|| username.clone());
        let discriminator = user
            .discriminator
            .as_ref()
            .map(|value| format!("{:04}", value.get()))
            .or_else(|| fallback.and_then(|value| value.discriminator.clone()));
        let avatar = user
            .avatar
            .as_ref()
            .map(ToString::to_string)
            .or_else(|| fallback.and_then(|value| value.avatar.clone()));
        let avatar_url = avatar
            .as_deref()
            .map(|hash| discord_avatar_url(&id, hash))
            .or_else(|| fallback.map(|value| value.avatar_url.clone()))
            .unwrap_or_else(|| default_avatar_url(&id, discriminator.as_deref()));
        let bot = user
            .bot
            .unwrap_or_else(|| fallback.map(|value| value.bot).unwrap_or(false));

        Self {
            id,
            username,
            display_name,
            global_name,
            discriminator,
            avatar,
            avatar_url,
            bot,
        }
    }

    pub fn offline(discord_user_id: &str, username: &str) -> Self {
        Self {
            id: discord_user_id.to_string(),
            username: username.to_string(),
            display_name: username.to_string(),
            global_name: None,
            discriminator: None,
            avatar: None,
            avatar_url: default_avatar_url(discord_user_id, None),
            bot: false,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActivitySummary {
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: u8,
    #[serde(skip_serializing)]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<ActivityAssetsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<ActivityTimestampsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<ActivityEmojiSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub created_at: u64,
}

impl ActivitySummary {
    fn from_activity(activity: &Activity) -> Self {
        Self {
            name: activity.name.clone(),
            activity_type: activity_type_to_code(&activity.kind),
            kind: activity_type_to_string(&activity.kind),
            state: activity.state.clone(),
            details: activity.details.clone(),
            application_id: activity
                .application_id
                .as_ref()
                .map(|id| id.get().to_string()),
            assets: activity
                .assets
                .as_ref()
                .map(ActivityAssetsSummary::from_assets),
            timestamps: activity
                .timestamps
                .as_ref()
                .map(ActivityTimestampsSummary::from_timestamps),
            emoji: activity
                .emoji
                .as_ref()
                .map(ActivityEmojiSummary::from_emoji),
            url: activity.url.as_ref().map(ToString::to_string),
            created_at: activity.created_at,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActivityAssetsSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

impl ActivityAssetsSummary {
    fn from_assets(assets: &ActivityAssets) -> Self {
        Self {
            large_image: assets.large_image.clone(),
            large_text: assets.large_text.clone(),
            small_image: assets.small_image.clone(),
            small_text: assets.small_text.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActivityTimestampsSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

impl ActivityTimestampsSummary {
    fn from_timestamps(timestamps: &ActivityTimestamps) -> Self {
        Self {
            start: timestamps.start,
            end: timestamps.end,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ActivityEmojiSummary {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub animated: bool,
}

impl ActivityEmojiSummary {
    fn from_emoji(emoji: &ActivityEmoji) -> Self {
        Self {
            name: emoji.name.clone(),
            id: emoji.id.as_ref().map(|id| id.get().to_string()),
            animated: emoji.animated.unwrap_or(false),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SpotifySummary {
    pub song: String,
    pub artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_art_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<ActivityTimestampsSummary>,
}

#[allow(dead_code)]
impl SpotifySummary {
    fn from_activity(activity: &Activity) -> Self {
        Self::from_summary(&ActivitySummary::from_activity(activity))
    }

    fn from_summary(activity: &ActivitySummary) -> Self {
        let album_art_url = activity
            .assets
            .as_ref()
            .and_then(|assets| assets.large_image.as_deref())
            .and_then(spotify_image_url);

        Self {
            song: activity
                .details
                .clone()
                .unwrap_or_else(|| activity.name.clone()),
            artist: activity
                .state
                .clone()
                .unwrap_or_else(|| "Unknown artist".to_string()),
            album: activity
                .assets
                .as_ref()
                .and_then(|assets| assets.large_text.clone()),
            album_art_url,
            timestamps: activity.timestamps.clone(),
        }
    }
}

pub fn spotify_image_url(value: &str) -> Option<String> {
    value
        .strip_prefix("spotify:")
        .map(|hash| format!("https://i.scdn.co/image/{hash}"))
        .or_else(|| {
            if value.starts_with("https://") || value.starts_with("http://") {
                Some(value.to_string())
            } else {
                None
            }
        })
}

#[allow(dead_code)]
fn is_spotify_activity(activity: &Activity) -> bool {
    if !matches!(activity.kind, ActivityType::Listening) {
        return false;
    }

    if activity.name.eq_ignore_ascii_case("Spotify") {
        return true;
    }

    activity
        .assets
        .as_ref()
        .and_then(|assets| assets.large_image.as_deref())
        .map(|value| value.starts_with("spotify:"))
        .unwrap_or(false)
}

fn is_spotify_summary(activity: &ActivitySummary) -> bool {
    if activity.kind != "listening" {
        return false;
    }

    if activity.name.eq_ignore_ascii_case("Spotify") {
        return true;
    }

    activity
        .assets
        .as_ref()
        .and_then(|assets| assets.large_image.as_deref())
        .map(|value| value.starts_with("spotify:"))
        .unwrap_or(false)
}

fn online_status_to_string(status: &OnlineStatus) -> String {
    match status {
        OnlineStatus::Online => "online",
        OnlineStatus::Idle => "idle",
        OnlineStatus::DoNotDisturb => "dnd",
        OnlineStatus::Invisible => "offline",
        OnlineStatus::Offline => "offline",
        _ => "unknown",
    }
    .to_string()
}

fn activity_type_to_code(kind: &ActivityType) -> u8 {
    match kind {
        ActivityType::Playing => 0,
        ActivityType::Streaming => 1,
        ActivityType::Listening => 2,
        ActivityType::Watching => 3,
        ActivityType::Custom => 4,
        ActivityType::Competing => 5,
        ActivityType::Unknown(value) => *value,
        _ => 255,
    }
}

fn activity_type_to_string(kind: &ActivityType) -> String {
    match kind {
        ActivityType::Playing => "playing",
        ActivityType::Streaming => "streaming",
        ActivityType::Listening => "listening",
        ActivityType::Watching => "watching",
        ActivityType::Custom => "custom",
        ActivityType::Competing => "competing",
        _ => "unknown",
    }
    .to_string()
}

fn discord_avatar_url(user_id: &str, hash: &str) -> String {
    let extension = if hash.starts_with("a_") { "gif" } else { "png" };
    format!("https://cdn.discordapp.com/avatars/{user_id}/{hash}.{extension}?size=256")
}

fn default_avatar_url(user_id: &str, discriminator: Option<&str>) -> String {
    let index = discriminator
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value % 5)
        .unwrap_or_else(|| {
            user_id
                .parse::<u64>()
                .map(|value| (value >> 22) % 6)
                .unwrap_or(0)
        });

    format!("https://cdn.discordapp.com/embed/avatars/{index}.png")
}

fn non_empty_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn non_empty_non_id(value: Option<&str>, user_id: &str) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != user_id)
        .map(|value| value.to_string())
}
