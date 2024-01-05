use anyhow::Context as _;
use serenity::all::{ChannelId, CommandDataOption, Context, GetMessages};

use crate::utils::CommandResult;

pub async fn delete(
    options: &[CommandDataOption],
    channel: ChannelId,
    ctx: &Context,
) -> CommandResult {
    let amount = options
        .get(0)
        .and_then(|arg| arg.value.as_i64())
        .and_then(|x| u8::try_from(x).ok())
        .context("Missing argument")?;
    let messages = channel.messages(ctx, GetMessages::new().limit(amount)).await?;
    channel.delete_messages(ctx, messages).await?;
    Ok("Now everyone knows you're censoring them!".into())
}
