use std::num::NonZeroU64;

use poise::serenity_prelude as serenity;

use crate::utils::Context;

#[poise::command(
    slash_command,
    subcommands("id_user", "id_role", "id_channel", "id_message", "id_server")
)]
#[allow(clippy::unused_async)]
pub async fn id(_: Context<'_>) -> anyhow::Result<()> {
    Ok(())
}
#[poise::command(slash_command, rename = "user")]
pub async fn id_user(ctx: Context<'_>, user: serenity::UserId) -> anyhow::Result<()> {
    generic_id(ctx, user.into()).await
}
#[poise::command(slash_command, rename = "role")]
pub async fn id_role(ctx: Context<'_>, role: serenity::RoleId) -> anyhow::Result<()> {
    generic_id(ctx, role.into()).await
}
#[poise::command(slash_command, rename = "channel")]
pub async fn id_channel(ctx: Context<'_>, channel: serenity::ChannelId) -> anyhow::Result<()> {
    generic_id(ctx, channel.into()).await
}
#[poise::command(slash_command, rename = "message")]
pub async fn id_message(ctx: Context<'_>, message: serenity::MessageId) -> anyhow::Result<()> {
    generic_id(ctx, message.into()).await
}

pub async fn generic_id(ctx: Context<'_>, id: NonZeroU64) -> anyhow::Result<()> {
    ctx.reply(format!("{id}")).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "server")]
pub async fn id_server(ctx: Context<'_>) -> anyhow::Result<()> {
    if let Some(id) = ctx.guild_id() {
        ctx.reply(format!("{id}")).await?;
    } else {
        ctx.reply("This can only be used on servers").await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn picture(ctx: Context<'_>, target: Option<serenity::User>) -> anyhow::Result<()> {
    if let Some(target) = target {
        ctx.reply(target.face()).await?;
    } else {
        ctx.reply(ctx.author().face()).await?;
    }
    Ok(())
}
