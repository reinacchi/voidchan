use serenity::{
    all::{
        ButtonStyle, CommandInteraction, ComponentInteraction, CreateActionRow, CreateButton,
        CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, Permissions,
    },
    client::Context,
};

use crate::app::error::AppError;

pub fn is_admin(permissions: Option<Permissions>) -> bool {
    permissions
        .map(|value| value.contains(Permissions::ADMINISTRATOR))
        .unwrap_or(false)
}

pub async fn respond(
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

pub async fn respond_component(
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

pub async fn respond_embed(
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

pub async fn respond_component_embed(
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

pub fn chunk_buttons(buttons: Vec<CreateButton>) -> Vec<Vec<CreateButton>> {
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

fn update_disabled(button: CreateButton, disabled: bool) -> CreateButton {
    button.disabled(disabled)
}

pub async fn update_component(
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

pub fn parse_owned_page_custom_id(custom_id: &str) -> Option<(&str, String, u64)> {
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

pub fn clamp_page(page: u64, total_pages: u64) -> u64 {
    if total_pages == 0 {
        0
    } else {
        page.min(total_pages.saturating_sub(1))
    }
}

pub fn total_pages(total_items: i64, page_size: u64) -> u64 {
    if total_items <= 0 {
        0
    } else {
        ((total_items as u64) + page_size - 1) / page_size
    }
}

pub fn owned_pagination_components(
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

pub fn admin_files_pagination_row(current_page: u64, total_pages: u64) -> Option<CreateActionRow> {
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
