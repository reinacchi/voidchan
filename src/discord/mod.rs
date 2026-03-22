use std::{collections::BTreeMap, sync::Arc};

use chrono::{DateTime, Utc};
use serenity::{
    all::{
        ButtonStyle, CommandDataOptionValue, CommandInteraction, CommandOptionType,
        ComponentInteraction, CreateActionRow, CreateButton, CreateCommand, CreateCommandOption,
        CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse,
        CreateInteractionResponseMessage, GatewayIntents, Guild, GuildId, Interaction, Member,
        OnlineStatus, Permissions, Presence, Ready, User as DiscordUser,
    },
    async_trait,
    client::{Client, Context, EventHandler},
    gateway::ShardManager,
    prelude::TypeMapKey,
};
use sqlx::{Row, query, query_as};

use crate::{
    app_state::AppState,
    error::AppError,
    models::User,
    utils::ids::{generate_api_token, is_valid_hex_colour, normalise_url_mode},
};

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub async fn run_bot(state: AppState, token: String, guild_id: u64) -> serenity::Result<()> {
    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_PRESENCES;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler {
            state,
            guild_id: GuildId::new(guild_id),
        })
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    client.start().await
}

struct Handler {
    state: AppState,
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let commands = vec![
            ping_command(),
            register_command(),
            config_command(),
            token_command(),
            profile_command(),
            files_command(),
            kv_command(),
            admin_users_command(),
            admin_files_command(),
            blacklist_command(),
            delete_file_command(),
        ];

        ctx.set_presence(None, OnlineStatus::DoNotDisturb);

        if let Err(err) = self.guild_id.set_commands(&ctx.http, commands).await {
            tracing::error!("failed to register discord commands: {err}");
        }
    }

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        if guild.id == self.guild_id {
            self.state.presence.sync_guild(&guild).await;
        }
    }

    async fn guild_member_addition(&self, _ctx: Context, new_member: Member) {
        if new_member.guild_id == self.guild_id {
            self.state.presence.upsert_member(&new_member).await;
        }
    }

    async fn guild_member_removal(
        &self,
        _ctx: Context,
        guild_id: GuildId,
        user: DiscordUser,
        _member_data_if_available: Option<Member>,
    ) {
        if guild_id == self.guild_id {
            self.state.presence.remove_user(user.id.get()).await;
        }
    }

    async fn presence_update(&self, _ctx: Context, new_data: Presence) {
        if new_data.guild_id == Some(self.guild_id) {
            self.state.presence.update_presence(&new_data).await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                if let Err(err) = self.handle_command(&ctx, &command).await {
                    tracing::error!("discord command error: {err:?}");
                    let _ = respond(
                        &ctx,
                        &command,
                        true,
                        format!("Something went wrong: {err:?}"),
                    )
                    .await;
                }
            }
            Interaction::Component(component) => {
                if let Err(err) = self.handle_component(&ctx, &component).await {
                    tracing::error!("discord component error: {err:?}");
                    let _ = respond_component(
                        &ctx,
                        &component,
                        true,
                        format!("Something went wrong: {err:?}"),
                    )
                    .await;
                }
            }
            _ => {}
        }
    }
}

impl Handler {
    async fn handle_command(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> Result<(), AppError> {
        match command.data.name.as_str() {
            "ping" => self.ping(ctx, command).await,
            "register" => self.register(ctx, command).await,
            "config" => self.config(ctx, command).await,
            "token" => self.token(ctx, command).await,
            "profile" => self.profile(ctx, command).await,
            "files" => self.files(ctx, command).await,
            "kv" => self.kv(ctx, command).await,
            "admin_users" => self.admin_users(ctx, command).await,
            "admin_files" => self.admin_files(ctx, command).await,
            "blacklist" => self.blacklist(ctx, command).await,
            "delete_file" => self.delete_file(ctx, command).await,
            _ => Ok(()),
        }
    }

    async fn handle_component(
        &self,
        ctx: &Context,
        component: &ComponentInteraction,
    ) -> Result<(), AppError> {
        if let Some(file_id) = component.data.custom_id.strip_prefix("file_detail:") {
            if !is_admin(component.member.as_ref().and_then(|m| m.permissions)) {
                respond_component(ctx, component, true, "Administrator permission required.")
                    .await?;
                return Ok(());
            }

            let row = sqlx::query(
                r#"
                SELECT f.id, f.original_name, f.mime_type, f.extension, f.size, f.uploader, f.created_at
                FROM files f
                WHERE f.id = ?
                LIMIT 1
                "#,
            )
            .bind(file_id)
            .fetch_optional(&self.state.db)
            .await?;

            let Some(row) = row else {
                respond_component(ctx, component, true, "File not found.").await?;
                return Ok(());
            };
            let id = row.try_get::<String, _>("id").unwrap_or_default();
            let extension = row
                .try_get::<String, _>("extension")
                .unwrap_or_else(|_| "bin".to_string());
            let embed = CreateEmbed::new()
                .title("File detail")
                .field("ID", format!("`{}`", id), true)
                .field(
                    "Name",
                    row.try_get::<Option<String>, _>("original_name")
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "(unknown)".to_string()),
                    true,
                )
                .field(
                    "Type",
                    row.try_get::<String, _>("mime_type").unwrap_or_default(),
                    true,
                )
                .field(
                    "Size",
                    format!(
                        "{} bytes",
                        row.try_get::<u64, _>("size").unwrap_or_default()
                    ),
                    true,
                )
                .field(
                    "Uploader",
                    row.try_get::<String, _>("uploader").unwrap_or_default(),
                    true,
                )
                .field(
                    "Created",
                    row.try_get::<chrono::NaiveDateTime, _>("created_at")
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    false,
                )
                .field(
                    "View URL",
                    format!(
                        "{}/v/{}.{}",
                        self.state.config.base_url.trim_end_matches('/'),
                        file_id,
                        extension
                    ),
                    false,
                )
                .field(
                    "Raw URL",
                    format!(
                        "{}/u/{}.{}",
                        self.state.config.base_url.trim_end_matches('/'),
                        file_id,
                        extension
                    ),
                    false,
                );
            respond_component_embed(ctx, component, true, embed).await?;
            return Ok(());
        }

        if let Some((kind, owner_id, page)) = parse_owned_page_custom_id(&component.data.custom_id)
        {
            if component.user.id.get().to_string() != owner_id {
                respond_component(
                    ctx,
                    component,
                    true,
                    "This paginator belongs to another user.",
                )
                .await?;
                return Ok(());
            }

            match kind {
                "files" => {
                    let (content, components) = self
                        .render_files_page_for_user(
                            owner_id
                                .parse::<u64>()
                                .map_err(|_| AppError::BadRequest("Invalid paginator owner."))?,
                            page,
                        )
                        .await?;
                    update_component(ctx, component, content, components).await?;
                    return Ok(());
                }
                "kv" => {
                    let (content, components) = self
                        .render_kv_page_for_user(
                            owner_id
                                .parse::<u64>()
                                .map_err(|_| AppError::BadRequest("Invalid paginator owner."))?,
                            page,
                        )
                        .await?;
                    update_component(ctx, component, content, components).await?;
                    return Ok(());
                }
                _ => {}
            }
        }

        if let Some(page) = component.data.custom_id.strip_prefix("page:admin_files:") {
            if !is_admin(component.member.as_ref().and_then(|m| m.permissions)) {
                respond_component(ctx, component, true, "Administrator permission required.")
                    .await?;
                return Ok(());
            }

            let page = page
                .parse::<u64>()
                .map_err(|_| AppError::BadRequest("Invalid page."))?;
            let (content, components) = self.render_admin_files_page(page).await?;
            update_component(ctx, component, content, components).await?;
            return Ok(());
        }

        Ok(())
    }

    async fn ping(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        let shard_manager = {
            let data = ctx.data.read().await;
            data.get::<ShardManagerContainer>()
                .cloned()
                .ok_or_else(|| AppError::Internal("ShardManager not found.".to_string()))?
        };

        let runners = shard_manager.runners.lock().await;

        let embed = match runners.get(&ctx.shard_id).and_then(|runner| runner.latency) {
            Some(latency) => CreateEmbed::new().title("Pong").field(
                "Gateway latency",
                format!("`{}` ms", latency.as_millis()),
                false,
            ),
            None => CreateEmbed::new()
                .title("Pong")
                .description("Latency is not available yet."),
        };

        respond_embed(ctx, command, true, embed).await?;
        Ok(())
    }

    async fn register(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        if command.guild_id.is_none() {
            respond(
                ctx,
                command,
                true,
                "This command can only be used in a guild.",
            )
            .await?;
            return Ok(());
        }

        self.state.presence.upsert_user(&command.user).await;

        let discord_id = command.user.id.get().to_string();
        let username = self
            .unique_username(&command.user.name, Some(&discord_id))
            .await?;

        let existing: Option<(u64, String)> =
            query_as("SELECT id, api_token FROM users WHERE discord_user_id = ? LIMIT 1")
                .bind(&discord_id)
                .fetch_optional(&self.state.db)
                .await?;

        if let Some((_user_id, api_token)) = existing {
            query("UPDATE users SET username = ? WHERE discord_user_id = ?")
                .bind(&username)
                .bind(&discord_id)
                .execute(&self.state.db)
                .await?;

            respond(
                ctx,
                command,
                true,
                register_success_message(&self.state.config.base_url, &username, &api_token, true),
            )
            .await?;
            return Ok(());
        }

        let api_token = generate_api_token();
        query(
            r#"
            INSERT INTO users
                (username, api_token, discord_user_id, preferred_url_mode, preferred_hex_colour, is_blacklisted, created_at)
            VALUES
                (?, ?, ?, 'v', '#7289da', FALSE, UTC_TIMESTAMP())
            "#,
        )
        .bind(&username)
        .bind(&api_token)
        .bind(&discord_id)
        .execute(&self.state.db)
        .await?;

        respond(
            ctx,
            command,
            true,
            register_success_message(&self.state.config.base_url, &username, &api_token, false),
        )
        .await?;
        Ok(())
    }

    async fn config(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        let Some(user) = self.get_registered_user(command.user.id.get()).await? else {
            respond(ctx, command, true, "Register first with `/register`.").await?;
            return Ok(());
        };

        let mut new_mode: Option<String> = None;
        let mut new_colour: Option<String> = None;

        for option in command.data.options.iter() {
            match (option.name.as_str(), &option.value) {
                ("url_mode", CommandDataOptionValue::String(value)) => {
                    let Some(normalized) = normalise_url_mode(value) else {
                        respond(ctx, command, true, "`url_mode` must be `v` or `u`.").await?;
                        return Ok(());
                    };
                    new_mode = Some(normalized.to_string());
                }
                ("hex_colour", CommandDataOptionValue::String(value)) => {
                    if !is_valid_hex_colour(value) {
                        respond(ctx, command, true, "`hex_colour` must look like `#7289da`.")
                            .await?;
                        return Ok(());
                    }
                    new_colour = Some(value.to_string());
                }
                _ => {}
            }
        }

        let next_mode = new_mode.unwrap_or_else(|| user.preferred_url_mode.clone());
        let next_colour = new_colour.unwrap_or_else(|| user.preferred_hex_colour.clone());

        query("UPDATE users SET preferred_url_mode = ?, preferred_hex_colour = ? WHERE id = ?")
            .bind(&next_mode)
            .bind(&next_colour)
            .bind(user.id)
            .execute(&self.state.db)
            .await?;

        respond_embed(
            ctx,
            command,
            true,
            CreateEmbed::new()
                .title("Configuration updated")
                .field("URL mode", format!("`/{}`", next_mode), true)
                .field("Hex colour", format!("`{}`", next_colour), true),
        )
        .await?;
        Ok(())
    }

    async fn token(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        let Some(user) = self.get_registered_user(command.user.id.get()).await? else {
            respond(ctx, command, true, "Register first with `/register`.").await?;
            return Ok(());
        };

        let mut action = "view".to_string();
        for option in command.data.options.iter() {
            if option.name.as_str() == "action" {
                if let CommandDataOptionValue::String(value) = &option.value {
                    action = value.to_string();
                }
            }
        }

        if action == "regenerate" {
            let token = generate_api_token();
            query("UPDATE users SET api_token = ? WHERE id = ?")
                .bind(&token)
                .bind(user.id)
                .execute(&self.state.db)
                .await?;

            respond(
                ctx,
                command,
                true,
                format!("Your new API token is `{token}`"),
            )
            .await?;
        } else {
            respond(
                ctx,
                command,
                true,
                format!("Your current API token is `{}`", user.api_token),
            )
            .await?;
        }

        Ok(())
    }

    async fn profile(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        let Some(user) = self.get_registered_user(command.user.id.get()).await? else {
            respond(ctx, command, true, "Register first with `/register`.").await?;
            return Ok(());
        };
        let total_uploads: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE uploader_id = ?")
                .bind(user.id)
                .fetch_one(&self.state.db)
                .await?;
        let total_kv_entries: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_presence_kv WHERE user_id = ?")
                .bind(user.id)
                .fetch_one(&self.state.db)
                .await?;

        let account_creation = DateTime::<Utc>::from_naive_utc_and_offset(user.created_at, Utc);
        let embed = CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(format!("@{}", user.username)).icon_url(format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.png?size=256",
                    command.user.id,
                    command
                        .user
                        .avatar
                        .as_ref()
                        .map(|hash| hash.to_string())
                        .unwrap_or("0".into())
                )),
            )
            .title("Your Profile")
            .field(
                "VoidChan Account Creation",
                format!("<t:{}:F>", account_creation.timestamp()),
                false,
            )
            .field(
                "Preferred URL Mode",
                format!("`/{}`", user.preferred_url_mode),
                true,
            )
            .field(
                "Preferred Hex Colour",
                format!("`{}`", user.preferred_hex_colour),
                true,
            )
            .field(
                "Total Files",
                format!("```prolog\n{}```", total_uploads),
                false,
            )
            .field(
                "Total KV Entries",
                format!("```prolog\n{}```", total_kv_entries),
                false,
            )
            .thumbnail(format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png?size=256",
                command.user.id,
                command
                    .user
                    .avatar
                    .as_ref()
                    .map(|hash| hash.to_string())
                    .unwrap_or("0".into()),
            ))
            .footer(CreateEmbedFooter::new(format!(
                "ID: {}",
                user.discord_user_id.unwrap_or_default()
            )));

        respond_embed(ctx, command, true, embed).await?;
        Ok(())
    }

    async fn files(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        let Some(user) = self.get_registered_user(command.user.id.get()).await? else {
            respond(ctx, command, true, "Register first with `/register`.").await?;
            return Ok(());
        };

        let (embed, components) = self.render_files_page_for_user(user.id, 0).await?;

        command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(embed)
                        .ephemeral(true)
                        .components(components),
                ),
            )
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn kv(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        let Some(user) = self.get_registered_user(command.user.id.get()).await? else {
            respond(ctx, command, true, "Register first with `/register`.").await?;
            return Ok(());
        };

        let Some(option) = command.data.options.first() else {
            let (embed, components) = self.render_kv_page_for_user(user.id, 0).await?;
            command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .add_embed(embed)
                            .ephemeral(true)
                            .components(components),
                    ),
                )
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            return Ok(());
        };

        match option.name.as_str() {
            "set" => {
                let mut key = None::<String>;
                let mut value = None::<String>;

                if let CommandDataOptionValue::SubCommand(sub_options) = &option.value {
                    for sub_option in sub_options {
                        match (sub_option.name.as_str(), &sub_option.value) {
                            ("key", CommandDataOptionValue::String(v)) => key = Some(v.to_string()),
                            ("value", CommandDataOptionValue::String(v)) => {
                                value = Some(v.to_string())
                            }
                            _ => {}
                        }
                    }
                }

                let Some(key) = key else {
                    respond(ctx, command, true, "Missing `key`.").await?;
                    return Ok(());
                };

                let Some(value) = value else {
                    respond(ctx, command, true, "Missing `value`.").await?;
                    return Ok(());
                };

                validate_kv_key(&key)?;
                validate_kv_value(&value)?;
                ensure_kv_capacity(&self.state, user.id, &[key.clone()]).await?;

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
                .execute(&self.state.db)
                .await?;

                respond_embed(
                    ctx,
                    command,
                    true,
                    CreateEmbed::new()
                        .title("KV Entry Stored")
                        .field("Key", format!("`{key}`"), true)
                        .field(
                            "Length",
                            format!("`{}` character(s)", value.chars().count()),
                            true,
                        ),
                )
                .await?;
            }
            "get" => {
                let mut key = None::<String>;

                if let CommandDataOptionValue::SubCommand(sub_options) = &option.value {
                    for sub_option in sub_options {
                        if let ("key", CommandDataOptionValue::String(v)) =
                            (sub_option.name.as_str(), &sub_option.value)
                        {
                            key = Some(v.to_string());
                        }
                    }
                }

                if let Some(key) = key {
                    validate_kv_key(&key)?;

                    let row = sqlx::query(
                        r#"
                        SELECT kv_value
                        FROM user_presence_kv
                        WHERE user_id = ? AND kv_key = ?
                        LIMIT 1
                        "#,
                    )
                    .bind(user.id)
                    .bind(&key)
                    .fetch_optional(&self.state.db)
                    .await?;

                    if let Some(row) = row {
                        let value = row.try_get::<String, _>("kv_value").unwrap_or_default();
                        let value_len = value.chars().count();
                        let display_value = if value_len > 1000 {
                            format!("{}...", value.chars().take(1000).collect::<String>())
                        } else {
                            value
                        };

                        respond_embed(
                            ctx,
                            command,
                            true,
                            CreateEmbed::new()
                                .title("KV Entry")
                                .field("Key", format!("`{key}`"), true)
                                .field("Length", format!("`{value_len}` character(s)"), true)
                                .field(
                                    "Value",
                                    format!(
                                        "```
{display_value}
```"
                                    ),
                                    false,
                                ),
                        )
                        .await?;
                    } else {
                        respond_embed(
                            ctx,
                            command,
                            true,
                            CreateEmbed::new()
                                .title("KV Entry Not Found")
                                .description(format!("No KV entry found for `{key}`.")),
                        )
                        .await?;
                    }
                } else {
                    let (embed, components) = self.render_kv_page_for_user(user.id, 0).await?;
                    command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .add_embed(embed)
                                    .ephemeral(true)
                                    .components(components),
                            ),
                        )
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;
                }
            }
            "clear" => {
                let result = sqlx::query("DELETE FROM user_presence_kv WHERE user_id = ?")
                    .bind(user.id)
                    .execute(&self.state.db)
                    .await?;

                respond_embed(
                    ctx,
                    command,
                    true,
                    CreateEmbed::new()
                        .title("KV Entry Cleared")
                        .description(format!(
                            "Cleared `{}` KV entr{}.",
                            result.rows_affected(),
                            if result.rows_affected() == 1 {
                                "y"
                            } else {
                                "ies"
                            }
                        )),
                )
                .await?;
            }
            "export" => {
                let rows = sqlx::query(
                    r#"
                    SELECT kv_key, kv_value
                    FROM user_presence_kv
                    WHERE user_id = ?
                    ORDER BY kv_key ASC
                    "#,
                )
                .bind(user.id)
                .fetch_all(&self.state.db)
                .await?;

                if rows.is_empty() {
                    respond(ctx, command, true, "You have no KV entries yet.").await?;
                    return Ok(());
                }

                let mut payload = BTreeMap::new();
                for row in rows {
                    payload.insert(
                        row.try_get::<String, _>("kv_key").unwrap_or_default(),
                        row.try_get::<String, _>("kv_value").unwrap_or_default(),
                    );
                }

                let json = serde_json::to_string_pretty(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize KV export: {e}"))
                })?;
                let wrapped = format!("```json\n{}\n```", json);

                if wrapped.chars().count() > 1900 {
                    respond(
                        ctx,
                        command,
                        true,
                        format!(
                            "KV export is too large to fit in a Discord message ({} characters).",
                            wrapped.chars().count()
                        ),
                    )
                    .await?;
                } else {
                    respond(ctx, command, true, wrapped).await?;
                }
            }
            "delete" => {
                let mut key = None::<String>;

                if let CommandDataOptionValue::SubCommand(sub_options) = &option.value {
                    for sub_option in sub_options {
                        if let ("key", CommandDataOptionValue::String(v)) =
                            (sub_option.name.as_str(), &sub_option.value)
                        {
                            key = Some(v.to_string());
                        }
                    }
                }

                let Some(key) = key else {
                    respond(ctx, command, true, "Missing `key`.").await?;
                    return Ok(());
                };

                validate_kv_key(&key)?;

                let result =
                    sqlx::query("DELETE FROM user_presence_kv WHERE user_id = ? AND kv_key = ?")
                        .bind(user.id)
                        .bind(&key)
                        .execute(&self.state.db)
                        .await?;

                if result.rows_affected() == 0 {
                    respond_embed(
                        ctx,
                        command,
                        true,
                        CreateEmbed::new()
                            .title("KV Entry Not Found")
                            .description(format!("No KV entry found for `{key}`.")),
                    )
                    .await?;
                } else {
                    respond_embed(
                        ctx,
                        command,
                        true,
                        CreateEmbed::new()
                            .title("KV Entry Deleted")
                            .description(format!("Deleted KV entry `{key}`.")),
                    )
                    .await?;
                }
            }
            _ => {
                respond(ctx, command, true, "Use `/kv get [key]`, `/kv set <key> <value>`, `/kv delete <key>`, `/kv clear`, or `/kv export`." ).await?;
            }
        }

        Ok(())
    }

    async fn admin_users(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> Result<(), AppError> {
        if !is_admin(command.member.as_ref().and_then(|m| m.permissions)) {
            respond(ctx, command, true, "Administrator permission required.").await?;
            return Ok(());
        }

        let rows = sqlx::query(
            r#"
            SELECT u.username, u.discord_user_id, u.preferred_url_mode, u.preferred_hex_colour, u.is_blacklisted,
                   COUNT(f.id) AS total_uploads
            FROM users u
            LEFT JOIN files f ON f.uploader_id = u.id
            GROUP BY u.id
            ORDER BY u.created_at DESC
            LIMIT 25
            "#,
        )
        .fetch_all(&self.state.db)
        .await?;

        let mut lines = Vec::new();
        for row in rows {
            lines.push(format!(
                "• `{}` | Discord: `{}` | mode `/{}` | colour `{}` | uploads `{}` | blacklisted `{}`",
                row.try_get::<String, _>("username").unwrap_or_default(),
                row.try_get::<Option<String>, _>("discord_user_id").ok().flatten().unwrap_or_else(|| "-".to_string()),
                row.try_get::<String, _>("preferred_url_mode").unwrap_or_else(|_| "v".to_string()),
                row.try_get::<String, _>("preferred_hex_colour").unwrap_or_else(|_| "#7289da".to_string()),
                row.try_get::<i64, _>("total_uploads").unwrap_or_default(),
                row.try_get::<bool, _>("is_blacklisted").unwrap_or(false),
            ));
        }

        respond_embed(
            ctx,
            command,
            true,
            CreateEmbed::new()
                .title("Users")
                .description(lines.join("\n")),
        )
        .await?;
        Ok(())
    }

    async fn admin_files(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> Result<(), AppError> {
        if !is_admin(command.member.as_ref().and_then(|m| m.permissions)) {
            respond(ctx, command, true, "Administrator permission required.").await?;
            return Ok(());
        }

        let (embed, components) = self.render_admin_files_page(0).await?;

        command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .add_embed(embed)
                        .ephemeral(true)
                        .components(components),
                ),
            )
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn blacklist(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        if !is_admin(command.member.as_ref().and_then(|m| m.permissions)) {
            respond(ctx, command, true, "Administrator permission required.").await?;
            return Ok(());
        }

        let mut discord_user_id = None::<String>;
        let mut blacklisted = None::<bool>;
        for option in command.data.options.iter() {
            match (option.name.as_str(), &option.value) {
                ("discord_user_id", CommandDataOptionValue::String(v)) => {
                    discord_user_id = Some(v.to_string())
                }
                ("blacklisted", CommandDataOptionValue::Boolean(v)) => blacklisted = Some(*v),
                _ => {}
            }
        }

        let Some(discord_user_id) = discord_user_id else {
            respond(ctx, command, true, "Missing `discord_user_id`.").await?;
            return Ok(());
        };
        let blacklisted = blacklisted.unwrap_or(true);

        let result = query("UPDATE users SET is_blacklisted = ? WHERE discord_user_id = ?")
            .bind(blacklisted)
            .bind(&discord_user_id)
            .execute(&self.state.db)
            .await?;

        if result.rows_affected() == 0 {
            respond(ctx, command, true, "User not found.").await?;
            return Ok(());
        }

        respond_embed(
            ctx,
            command,
            true,
            CreateEmbed::new()
                .title("Blacklist updated")
                .field("Discord user ID", format!("`{discord_user_id}`"), true)
                .field("Blacklisted", format!("`{blacklisted}`"), true),
        )
        .await?;
        Ok(())
    }

    async fn delete_file(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
    ) -> Result<(), AppError> {
        if !is_admin(command.member.as_ref().and_then(|m| m.permissions)) {
            respond(ctx, command, true, "Administrator permission required.").await?;
            return Ok(());
        }

        let mut file_id = None::<String>;
        for option in command.data.options.iter() {
            if option.name.as_str() == "file_id" {
                if let CommandDataOptionValue::String(v) = &option.value {
                    file_id = Some(v.to_string());
                }
            }
        }

        let Some(file_id) = file_id else {
            respond(ctx, command, true, "Missing `file_id`.").await?;
            return Ok(());
        };

        let file = sqlx::query("SELECT object_key FROM files WHERE id = ? LIMIT 1")
            .bind(&file_id)
            .fetch_optional(&self.state.db)
            .await?;

        let Some(file) = file else {
            respond(ctx, command, true, "File not found.").await?;
            return Ok(());
        };

        let object_key: String = file.try_get("object_key").unwrap_or_default();

        self.state
            .s3
            .delete_object()
            .bucket(&self.state.config.r2_bucket)
            .key(&object_key)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete object from R2: {e}")))?;

        query("DELETE FROM files WHERE id = ?")
            .bind(&file_id)
            .execute(&self.state.db)
            .await?;

        respond_embed(
            ctx,
            command,
            true,
            CreateEmbed::new()
                .title("File deleted")
                .description(format!("Deleted file `{file_id}`.")),
        )
        .await?;
        Ok(())
    }

    async fn render_files_page_for_user(
        &self,
        user_id: u64,
        page: u64,
    ) -> Result<(CreateEmbed, Vec<CreateActionRow>), AppError> {
        let total_files: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE uploader_id = ?")
                .bind(user_id)
                .fetch_one(&self.state.db)
                .await?;

        let page_size = 10_u64;
        let total_pages = total_pages(total_files, page_size);
        if total_pages == 0 {
            return Ok((
                CreateEmbed::new()
                    .title("Your uploaded files")
                    .description("You have not uploaded any files yet."),
                Vec::new(),
            ));
        }

        let current_page = clamp_page(page, total_pages);
        let offset = (current_page * page_size) as i64;
        let rows = sqlx::query(
            r#"
            SELECT id, original_name, extension, created_at
            FROM files
            WHERE uploader_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&self.state.db)
        .await?;

        let mut lines = Vec::new();
        for row in rows {
            let id: String = row.try_get("id").unwrap_or_default();
            let extension = row.try_get::<String, _>("extension").unwrap_or_default();
            let name = row
                .try_get::<Option<String>, _>("original_name")
                .ok()
                .flatten()
                .unwrap_or_else(|| format!("{}.{}", id, extension));
            let created = row
                .try_get::<chrono::NaiveDateTime, _>("created_at")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let url = format!(
                "{}/v/{}.{}",
                self.state.config.base_url.trim_end_matches('/'),
                id,
                extension
            );
            lines.push(format!(
                "• `{}` — {}
  URL: {}
  Uploaded: {}",
                id, name, url, created
            ));
        }

        Ok((
            CreateEmbed::new()
                .title("Your uploaded files")
                .description(lines.join("\n\n"))
                .footer(serenity::builder::CreateEmbedFooter::new(format!(
                    "Page {}/{}",
                    current_page + 1,
                    total_pages
                ))),
            owned_pagination_components("files", user_id, current_page, total_pages),
        ))
    }

    async fn render_kv_page_for_user(
        &self,
        user_id: u64,
        page: u64,
    ) -> Result<(CreateEmbed, Vec<CreateActionRow>), AppError> {
        let total_rows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_presence_kv WHERE user_id = ?")
                .bind(user_id)
                .fetch_one(&self.state.db)
                .await?;

        let page_size = 20_u64;
        let total_pages = total_pages(total_rows, page_size);
        if total_pages == 0 {
            return Ok((
                CreateEmbed::new()
                    .title("Your KV Keys")
                    .description("You have no KV entries yet."),
                Vec::new(),
            ));
        }

        let current_page = clamp_page(page, total_pages);
        let offset = (current_page * page_size) as i64;
        let rows = sqlx::query(
            r#"
            SELECT kv_key
            FROM user_presence_kv
            WHERE user_id = ?
            ORDER BY kv_key ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&self.state.db)
        .await?;

        let mut lines = Vec::new();
        for row in rows {
            lines.push(format!(
                "• `{}`",
                row.try_get::<String, _>("kv_key").unwrap_or_default()
            ));
        }

        Ok((
            CreateEmbed::new()
                .title("Your KV Keys")
                .description(lines.join("\n"))
                .footer(serenity::builder::CreateEmbedFooter::new(format!(
                    "Page {}/{} • Total {}",
                    current_page + 1,
                    total_pages,
                    total_rows
                ))),
            owned_pagination_components("kv", user_id, current_page, total_pages),
        ))
    }

    async fn render_admin_files_page(
        &self,
        page: u64,
    ) -> Result<(CreateEmbed, Vec<CreateActionRow>), AppError> {
        let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
            .fetch_one(&self.state.db)
            .await?;

        let page_size = 10_u64;
        let total_pages = total_pages(total_files, page_size);
        if total_pages == 0 {
            return Ok((
                CreateEmbed::new()
                    .title("Recent Uploaded Files")
                    .description("No uploaded files found."),
                Vec::new(),
            ));
        }

        let current_page = clamp_page(page, total_pages);
        let offset = (current_page * page_size) as i64;
        let rows = sqlx::query(
            r#"
            SELECT id, original_name, extension, uploader
            FROM files
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&self.state.db)
        .await?;

        let mut content = Vec::new();
        let mut buttons: Vec<CreateButton> = Vec::new();
        for row in rows {
            let id: String = row.try_get("id").unwrap_or_default();
            let label = row
                .try_get::<Option<String>, _>("original_name")
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    format!(
                        "{}.{}",
                        id,
                        row.try_get::<String, _>("extension").unwrap_or_default()
                    )
                });
            content.push(format!(
                "• `{}` — {} — uploader `{}`",
                id,
                label,
                row.try_get::<String, _>("uploader").unwrap_or_default()
            ));
            buttons.push(
                CreateButton::new(format!("file_detail:{id}"))
                    .label(format!("Detail {id}"))
                    .style(ButtonStyle::Primary),
            );
        }

        let mut components = chunk_buttons(buttons)
            .into_iter()
            .map(CreateActionRow::Buttons)
            .collect::<Vec<_>>();
        if let Some(nav) = admin_files_pagination_row(current_page, total_pages) {
            components.push(nav);
        }

        Ok((
            CreateEmbed::new()
                .title("Recent Uploaded Files")
                .description(content.join(
                    "
",
                ))
                .footer(serenity::builder::CreateEmbedFooter::new(format!(
                    "Page {}/{}",
                    current_page + 1,
                    total_pages
                ))),
            components,
        ))
    }

    async fn get_registered_user(&self, discord_user_id: u64) -> Result<Option<User>, AppError> {
        query_as::<_, User>(
            r#"
            SELECT id, username, api_token, discord_user_id, preferred_url_mode, preferred_hex_colour, is_blacklisted, created_at
            FROM users
            WHERE discord_user_id = ?
            LIMIT 1
            "#,
        )
        .bind(discord_user_id.to_string())
        .fetch_optional(&self.state.db)
        .await
        .map_err(Into::into)
    }

    async fn unique_username(
        &self,
        preferred: &str,
        discord_user_id: Option<&str>,
    ) -> Result<String, AppError> {
        let base = preferred
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .take(32)
            .collect::<String>();
        let fallback = if base.is_empty() {
            "discord-user".to_string()
        } else {
            base
        };

        let row = sqlx::query("SELECT id, discord_user_id FROM users WHERE username = ? LIMIT 1")
            .bind(&fallback)
            .fetch_optional(&self.state.db)
            .await?;

        if let Some(row) = row {
            let existing_discord = row
                .try_get::<Option<String>, _>("discord_user_id")
                .ok()
                .flatten();
            if existing_discord.as_deref() == discord_user_id {
                return Ok(fallback);
            }
            let suffix = discord_user_id.unwrap_or("user");
            let suffix = &suffix[suffix.len().saturating_sub(4)..];
            return Ok(format!("{}-{}", fallback, suffix));
        }

        Ok(fallback)
    }
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
        .collect::<std::collections::HashSet<_>>();

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

fn is_admin(permissions: Option<Permissions>) -> bool {
    permissions
        .map(|value| value.contains(Permissions::ADMINISTRATOR))
        .unwrap_or(false)
}

async fn respond(
    ctx: &Context,
    command: &CommandInteraction,
    ephemeral: bool,
    content: impl Into<String>,
) -> Result<(), AppError> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content.into())
                    .ephemeral(ephemeral),
            ),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

async fn respond_component(
    ctx: &Context,
    component: &ComponentInteraction,
    ephemeral: bool,
    content: impl Into<String>,
) -> Result<(), AppError> {
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content.into())
                    .ephemeral(ephemeral),
            ),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

async fn respond_embed(
    ctx: &Context,
    command: &CommandInteraction,
    ephemeral: bool,
    embed: CreateEmbed,
) -> Result<(), AppError> {
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .add_embed(embed)
                    .ephemeral(ephemeral),
            ),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

async fn respond_component_embed(
    ctx: &Context,
    component: &ComponentInteraction,
    ephemeral: bool,
    embed: CreateEmbed,
) -> Result<(), AppError> {
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .add_embed(embed)
                    .ephemeral(ephemeral),
            ),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn chunk_buttons(buttons: Vec<CreateButton>) -> Vec<Vec<CreateButton>> {
    let mut rows = Vec::new();
    let mut current = Vec::new();

    for button in buttons {
        current.push(button);
        if current.len() == 5 {
            rows.push(current);
            current = Vec::new();
        }
    }

    if !current.is_empty() {
        rows.push(current);
    }

    rows
}

fn ping_command() -> CreateCommand {
    CreateCommand::new("ping")
        .description("Show the bot gateway latency")
        .dm_permission(false)
}

fn register_success_message(
    base_url: &str,
    username: &str,
    api_token: &str,
    already_registered: bool,
) -> String {
    let status_line = if already_registered {
        format!("You are already registered as `{username}`.")
    } else {
        format!("Registered successfully. Username: `{username}`")
    };

    format!(
        r#"{status_line}
API token: `{api_token}`

Get started:
1. Copy the JSON below.
2. Replace `YOUR_API_TOKEN` with your token.
3. In ShareX, go to `Custom uploader settings...` -> `Import` -> `From clipboard`.
4. Set this uploader as both your **Image uploader** and your **File uploader** in `Destinations`.

```json
{{
  "Version": "19.0.0",
  "Name": "VoidChan",
  "DestinationType": "ImageUploader, FileUploader, TextUploader",
  "RequestMethod": "POST",
  "RequestURL": "{}/api/providers/sharex",
  "Body": "MultipartFormData",
  "FileFormName": "file",
  "Headers": {{
    "Authorization": "YOUR_API_TOKEN"
  }},
  "URL": "{{json:url}}",
  "ErrorMessage": "{{json:message}}"
}}
```"#,
        base_url.trim_end_matches('/'),
    )
}

fn register_command() -> CreateCommand {
    CreateCommand::new("register")
        .description("Register your Discord account and get setup instructions")
        .dm_permission(false)
}

fn config_command() -> CreateCommand {
    CreateCommand::new("config")
        .description("Update your CDN preferences")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "url_mode",
                "Preferred copied URL mode",
            )
            .required(false)
            .add_string_choice("View URL (/v)", "v")
            .add_string_choice("Raw URL (/u)", "u"),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "hex_colour",
                "Preferred embed/meta hex colour like #7289da",
            )
            .required(false),
        )
        .dm_permission(false)
}

fn token_command() -> CreateCommand {
    CreateCommand::new("token")
        .description("View or regenerate your API token")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "action",
                "Choose whether to view or regenerate",
            )
            .required(true)
            .add_string_choice("View", "view")
            .add_string_choice("Regenerate", "regenerate"),
        )
        .dm_permission(false)
}

fn profile_command() -> CreateCommand {
    CreateCommand::new("profile")
        .description("View your CDN profile")
        .dm_permission(false)
}

fn files_command() -> CreateCommand {
    CreateCommand::new("files")
        .description("List your uploaded files")
        .dm_permission(false)
}

fn kv_command() -> CreateCommand {
    CreateCommand::new("kv")
        .description("Manage your presence KV entries")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "get",
                "Get a KV value or list all keys",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "key", "KV key")
                    .required(false),
            ),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "set", "Set a KV value")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::String, "key", "KV key")
                        .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::String, "value", "KV value")
                        .required(true),
                ),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "delete", "Delete a KV value")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::String, "key", "KV key")
                        .required(true),
                ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "clear",
            "Delete all KV entries",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "export",
            "Export all KV entries as JSON",
        ))
        .dm_permission(false)
}

fn admin_users_command() -> CreateCommand {
    CreateCommand::new("admin_users")
        .description("Admin: list users")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}

fn admin_files_command() -> CreateCommand {
    CreateCommand::new("admin_files")
        .description("Admin: list uploaded files")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}

fn blacklist_command() -> CreateCommand {
    CreateCommand::new("blacklist")
        .description("Admin: blacklist or unblacklist a user")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "discord_user_id",
                "Discord user ID",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Boolean,
                "blacklisted",
                "Whether the user should be blacklisted",
            )
            .required(true),
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}

fn delete_file_command() -> CreateCommand {
    CreateCommand::new("delete_file")
        .description("Admin: delete an uploaded file")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "file_id", "File ID to delete")
                .required(true),
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}

fn update_disabled(button: CreateButton, disabled: bool) -> CreateButton {
    button.disabled(disabled)
}

async fn update_component(
    ctx: &Context,
    component: &ComponentInteraction,
    embed: CreateEmbed,
    components: Vec<CreateActionRow>,
) -> Result<(), AppError> {
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .embeds(vec![embed])
                    .components(components),
            ),
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn parse_owned_page_custom_id(custom_id: &str) -> Option<(&str, String, u64)> {
    let mut parts = custom_id.split(':');
    let prefix = parts.next()?;
    let kind = parts.next()?;
    let owner_id = parts.next()?.to_string();
    let page = parts.next()?.parse::<u64>().ok()?;
    if prefix != "page" || parts.next().is_some() {
        return None;
    }
    Some((kind, owner_id, page))
}

fn clamp_page(page: u64, total_pages: u64) -> u64 {
    if total_pages == 0 {
        0
    } else {
        page.min(total_pages.saturating_sub(1))
    }
}

fn total_pages(total_items: i64, page_size: u64) -> u64 {
    if total_items <= 0 {
        0
    } else {
        ((total_items as u64) + page_size - 1) / page_size
    }
}

fn owned_pagination_components(
    kind: &str,
    user_id: u64,
    current_page: u64,
    total_pages: u64,
) -> Vec<CreateActionRow> {
    if total_pages <= 1 {
        return Vec::new();
    }

    vec![CreateActionRow::Buttons(vec![
        update_disabled(
            CreateButton::new(format!(
                "page:{kind}:{user_id}:{}",
                current_page.saturating_sub(1)
            ))
            .label("Previous")
            .style(ButtonStyle::Secondary),
            current_page == 0,
        ),
        update_disabled(
            CreateButton::new(format!("page:{kind}:{user_id}:{}", current_page + 1))
                .label("Next")
                .style(ButtonStyle::Secondary),
            current_page + 1 >= total_pages,
        ),
    ])]
}

fn admin_files_pagination_row(current_page: u64, total_pages: u64) -> Option<CreateActionRow> {
    if total_pages <= 1 {
        return None;
    }

    Some(CreateActionRow::Buttons(vec![
        update_disabled(
            CreateButton::new(format!(
                "page:admin_files:{}",
                current_page.saturating_sub(1)
            ))
            .label("Previous")
            .style(ButtonStyle::Secondary),
            current_page == 0,
        ),
        update_disabled(
            CreateButton::new(format!("page:admin_files:{}", current_page + 1))
                .label("Next")
                .style(ButtonStyle::Secondary),
            current_page + 1 >= total_pages,
        ),
    ]))
}
