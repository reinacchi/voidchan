use std::sync::Arc;

use serenity::{
    all::{
        ButtonStyle, CommandDataOptionValue, CommandInteraction, CommandOptionType,
        ComponentInteraction, CreateActionRow, CreateButton, CreateCommand, CreateCommandOption,
        CreateInteractionResponse, CreateInteractionResponseMessage, GatewayIntents, Guild, GuildId,
        Interaction, Member, OnlineStatus, Permissions, Presence, Ready,
        User as DiscordUser,
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
        if !is_admin(component.member.as_ref().and_then(|m| m.permissions)) {
            respond_component(ctx, component, true, "Administrator permission required.").await?;
            return Ok(());
        }

        if let Some(file_id) = component.data.custom_id.strip_prefix("file_detail:") {
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
            let message = format!(
                "**File detail**
ID: `{}`
Name: {}
Type: {}
Size: {} bytes
Uploader: {}
Created: {}
View: {}/v/{}.{}
Raw: {}/u/{}.{}",
                id,
                row.try_get::<Option<String>, _>("original_name")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "(unknown)".to_string()),
                row.try_get::<String, _>("mime_type").unwrap_or_default(),
                row.try_get::<u64, _>("size").unwrap_or_default(),
                row.try_get::<String, _>("uploader").unwrap_or_default(),
                row.try_get::<chrono::NaiveDateTime, _>("created_at")
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                self.state.config.base_url.trim_end_matches('/'),
                file_id,
                extension,
                self.state.config.base_url.trim_end_matches('/'),
                file_id,
                extension,
            );
            respond_component(ctx, component, true, message).await?;
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

        let content = match runners.get(&ctx.shard_id).and_then(|runner| runner.latency) {
            Some(latency) => format!("Pong! `{}` ms", latency.as_millis()),
            None => "Pong! Latency not available yet.".to_string(),
        };

        respond(ctx, command, true, content).await?;
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

        respond(
            ctx,
            command,
            true,
            format!(
                "Updated config. URL mode: `/{}` | Hex colour: `{}`",
                next_mode, next_colour
            ),
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

        respond(
            ctx,
            command,
            true,
            format!(
                "**Profile**\nUsername: `{}`\nPreferred URL mode: `/{}`\nPreferred hex colour: `{}`\nTotal files uploaded: `{}`",
                user.username, user.preferred_url_mode, user.preferred_hex_colour, total_uploads
            ),
        )
        .await?;
        Ok(())
    }

    async fn files(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
        let Some(user) = self.get_registered_user(command.user.id.get()).await? else {
            respond(ctx, command, true, "Register first with `/register`.").await?;
            return Ok(());
        };

        let rows = sqlx::query(
            r#"
            SELECT id, original_name, extension, created_at
            FROM files
            WHERE uploader_id = ?
            ORDER BY created_at DESC
            LIMIT 20
            "#,
        )
        .bind(user.id)
        .fetch_all(&self.state.db)
        .await?;

        if rows.is_empty() {
            respond(ctx, command, true, "You have not uploaded any files yet.").await?;
            return Ok(());
        }

        let mut lines = vec!["**Your uploaded files**".to_string()];
        for row in rows {
            let id: String = row.try_get("id").unwrap_or_default();
            let name = row
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
            let created = row
                .try_get::<chrono::NaiveDateTime, _>("created_at")
                .map(|v| v.to_string())
                .unwrap_or_default();
            let url = format!(
                "{}/v/{}.{}",
                self.state.config.base_url.trim_end_matches('/'),
                id,
                row.try_get::<String, _>("extension").unwrap_or_default()
            );
            lines.push(format!("• `{}` — {} — {}", id, name, url));
            lines.push(format!("  Uploaded: {}", created));
        }

        respond(ctx, command, true, lines.join("\n")).await?;
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

        let mut lines = vec!["**Users**".to_string()];
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

        respond(ctx, command, true, lines.join("\n")).await?;
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

        let rows = sqlx::query(
            r#"
            SELECT id, original_name, extension, uploader
            FROM files
            ORDER BY created_at DESC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.state.db)
        .await?;

        if rows.is_empty() {
            respond(ctx, command, true, "No uploaded files found.").await?;
            return Ok(());
        }

        let mut content = vec!["**Recent uploaded files**".to_string()];
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

        let rows = chunk_buttons(buttons)
            .into_iter()
            .map(CreateActionRow::Buttons)
            .collect::<Vec<_>>();

        command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content(content.join("\n"))
                        .ephemeral(true)
                        .components(rows),
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

        respond(
            ctx,
            command,
            true,
            format!("Updated blacklist for `{discord_user_id}` to `{blacklisted}`."),
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

        respond(ctx, command, true, format!("Deleted file `{file_id}`.")).await?;
        Ok(())
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
