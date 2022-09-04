use std::borrow::Cow;

use anyhow::{Context as _, Result};
use serenity::{
    model::{application::interaction::application_command::CommandDataOption, id::ChannelId},
    prelude::*,
};

pub async fn delete(
    options: &[CommandDataOption],
    channel: ChannelId,
    ctx: &Context,
) -> Result<Cow<'static, str>> {
    let amount = options
        .get(0)
        .and_then(|arg| arg.value.as_ref())
        .and_then(serde_json::Value::as_u64)
        .context("Missing argument")?;
    let messages = channel
        .messages(ctx, |retriever| retriever.limit(amount))
        .await?;
    channel.delete_messages(ctx, messages).await?;
    Ok("Now everyone knows you're censoring them!".into())
}
