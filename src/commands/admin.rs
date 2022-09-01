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
        .context("Get argument")?
        .value
        .as_ref()
        .context("Get argument")?
        .as_i64()
        .context("Argument was not a number (Delete Command)")?;
    if !(1..=100).contains(&amount) {
        return Ok("Please specify a valid amount of messages to delete (1-100).".into());
    }
    #[allow(clippy::cast_sign_loss)]
    let messages = match channel
        .messages(ctx, |retriever| retriever.limit(amount as u64))
        .await
    {
        Ok(x) => x,
        Err(_) => return Ok("Something went wrong".into()),
    };
    drop(channel.delete_messages(ctx, messages).await);
    Ok("Now everyone knows you're censoring them!".into())
}
