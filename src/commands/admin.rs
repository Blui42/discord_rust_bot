use crate::data::prefix;
use anyhow::{Context as CTX, Result};
use serenity::{model::channel::Message, prelude::*};
use tokio::time::{sleep, Duration};

#[cfg(feature = "legacy_commands")]
pub async fn kick(ctx: Context, msg: Message) -> Result<()> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member lacks permission
    if !msg
        .member(&ctx.http)
        .await?
        .permissions(&ctx)?
        .kick_members()
    {
        msg.channel_id
            .say(&ctx.http, "You don't have permission to kick")
            .await?;
        return Ok(());
    }
    if msg.mentions.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Noone was mentioned, so noone was kicked")
            .await?;
        return Ok(());
    }
    if msg.mentions_me(&ctx).await.unwrap_or(true) {
        msg.channel_id.say(&ctx.http, "I won't kick myself").await?;
        return Ok(());
    }
    let a = msg.guild(&ctx.cache).context("Unknown Cache Error")?;
    for i in &msg.mentions {
        if a.kick_with_reason(
            &ctx.http,
            i.id,
            &format!("Kicked by `{}`", msg.author.tag()),
        )
        .await
        .is_ok()
        {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("Kicked `{}`", i.tag()))
                .await
            {
                eprintln!("An Error occured: {}", why);
            }
        } else if let Err(why) = msg
            .channel_id
            .say(&ctx.http, format!("Can't kick `{}`", i.tag()))
            .await
        {
            eprintln!("An Error occured: {}", why);
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}
#[cfg(feature = "legacy_commands")]
pub async fn unban(ctx: Context, msg: Message) -> Result<()> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member lacks permission
    if !msg
        .member(&ctx.http)
        .await?
        .permissions(&ctx)?
        .ban_members()
    {
        msg.channel_id
            .say(&ctx.http, "You don't have permission to unban")
            .await?;
        return Ok(());
    }
    if msg.mentions.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Noone was mentioned, so noone was unbanned")
            .await?;
        return Ok(());
    }
    let a = msg.guild(&ctx.cache).context("Unknown Cache Error")?;
    for i in &msg.mentions {
        if a.unban(&ctx.http, i.id).await.is_ok() {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("Unbanned `{}`", i.tag()))
                .await
            {
                eprintln!("An Error occured: {}", why);
            }
        } else if let Err(why) = msg
            .channel_id
            .say(&ctx.http, format!("Can't unban `{}`", i.tag()))
            .await
        {
            eprintln!("An Error occured: {}", why);
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}
#[cfg(feature = "legacy_commands")]
pub async fn ban(ctx: Context, msg: Message) -> Result<()> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member lacks permission
    if !msg
        .member(&ctx.http)
        .await?
        .permissions(&ctx)?
        .ban_members()
    {
        msg.channel_id
            .say(&ctx.http, "You don't have permission to ban")
            .await?;
        return Ok(());
    }
    if msg.mentions.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Noone was mentioned, so noone was banned")
            .await?;
        return Ok(());
    }
    if msg.mentions_me(&ctx).await.unwrap_or(true) {
        msg.channel_id.say(&ctx.http, "I won't ban myself").await?;
        return Ok(());
    }
    let a = msg.guild(&ctx.cache).context("Unknown Cache Error")?;
    for i in &msg.mentions {
        if a.ban_with_reason(
            &ctx.http,
            i.id,
            0,
            &format!("Banned by `{}`", msg.author.tag()),
        )
        .await
        .is_ok()
        {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("Banned `{}`", i.tag()))
                .await
            {
                eprintln!("An Error occured: {}", why);
            }
        } else if let Err(why) = msg
            .channel_id
            .say(&ctx.http, format!("Can't ban `{}`", i.tag()))
            .await
        {
            eprintln!("An Error occured: {}", why);
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}
#[cfg(feature = "legacy_commands")]
pub async fn prefix(ctx: Context, msg: Message) -> Result<()> {
    let guild = if let Some(a) = msg.guild_id {
        a
    } else {
        // Ignore Private Messages
        return Ok(());
    };
    // if the member lacks permission
    if !msg
        .member(&ctx.http)
        .await?
        .permissions(&ctx)?
        .administrator()
    {
        msg.channel_id
            .say(
                &ctx.http,
                "Only administrators can change the server prefix",
            )
            .await?;
        return Ok(());
    }
    let new_prefix = msg.content.replace("`", "");
    if new_prefix.len() > 5 {
        msg.channel_id
            .say(&ctx.http, "Prefixes can't be longer than 5 symbols.")
            .await?;
        return Ok(());
    }
    prefix::set(guild, &ctx, &new_prefix).await;
    msg.channel_id
        .say(
            &ctx.http,
            format!("Successfully set prefix to `{}`", new_prefix),
        )
        .await?;
    Ok(())
}
#[cfg(feature = "legacy_commands")]
pub async fn delete(ctx: Context, msg: Message) -> Result<()> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member lacks permission
    if !msg
        .member(&ctx.http)
        .await?
        .permissions(&ctx.cache)?
        .manage_messages()
    {
        msg.channel_id
            .say(&ctx.http, "You can't delete messages")
            .await?;
        return Ok(());
    }

    let amount: u64 = msg.content.parse().unwrap_or(0);
    if amount == 0 {
        let error_msg = msg
            .channel_id
            .say(
                &ctx.http,
                "Please specify a valid amount of messages to delete (1-100)",
            )
            .await?;
        sleep(Duration::from_secs(3)).await;
        error_msg.delete(&ctx.http).await?;
        return Ok(());
    }
    let guild_channel = msg
        .channel(&ctx.http)
        .await?
        .guild()
        .context("Unknown Cache Error getting Guild")?;
    let messages = guild_channel
        .messages(&ctx.http, |retriever| retriever.limit(amount + 1))
        .await?;
    if guild_channel
        .delete_messages(&ctx.http, messages)
        .await
        .is_err()
    {
        let error_msg = msg
            .channel_id
            .say(
                &ctx.http,
                "Please specify a valid amount of messages to delete (1-100)",
            )
            .await?;
        sleep(Duration::from_secs(3)).await;
        error_msg.delete(&ctx.http).await?;
    }
    Ok(())
}
