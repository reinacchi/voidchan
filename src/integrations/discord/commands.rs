use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, Permissions};

pub fn ping_command() -> CreateCommand {
    CreateCommand::new("ping")
        .description("Show the bot gateway latency")
        .dm_permission(false)
}

pub fn register_success_message(
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

pub fn register_command() -> CreateCommand {
    CreateCommand::new("register")
        .description("Register your Discord account and get setup instructions")
        .dm_permission(false)
}

pub fn config_command() -> CreateCommand {
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

pub fn token_command() -> CreateCommand {
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

pub fn profile_command() -> CreateCommand {
    CreateCommand::new("profile")
        .description("View your CDN profile")
        .dm_permission(false)
}

pub fn files_command() -> CreateCommand {
    CreateCommand::new("files")
        .description("List your uploaded files")
        .dm_permission(false)
}

pub fn kv_command() -> CreateCommand {
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

pub fn admin_users_command() -> CreateCommand {
    CreateCommand::new("admin_users")
        .description("Admin: list users")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}

pub fn admin_files_command() -> CreateCommand {
    CreateCommand::new("admin_files")
        .description("Admin: list uploaded files")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}

pub fn blacklist_command() -> CreateCommand {
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

pub fn delete_file_command() -> CreateCommand {
    CreateCommand::new("delete_file")
        .description("Admin: delete an uploaded file")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "file_id", "File ID to delete")
                .required(true),
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .dm_permission(false)
}
