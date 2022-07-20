use super::stringify_error;
use crate::data::prefix;
use serenity::{model::channel::Message, prelude::*};
use tokio::time::{sleep, Duration};

#[cfg(feature = "legacy_commands")]
pub async fn kick(ctx: Context, msg: Message) -> Result<(), String> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member doesn't have the Admin permission
    if !msg
        .member(&ctx.http)
        .await
        .map_err(stringify_error)?
        .permissions(&ctx)
        .map_err(stringify_error)?
        .kick_members()
    {
        msg.channel_id
            .say(&ctx.http, "You don't have permission to kick")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    if msg.mentions.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Noone was mentioned, so noone was kicked")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    if msg.mentions_me(&ctx).await.unwrap_or(true) {
        msg.channel_id
            .say(&ctx.http, "I won't kick myself")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    let a = msg.guild(&ctx.cache).ok_or("Unknown Cache Error")?;
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
pub async fn unban(ctx: Context, msg: Message) -> Result<(), String> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member doesn't have the Admin permission
    if !msg
        .member(&ctx.http)
        .await
        .map_err(stringify_error)?
        .permissions(&ctx)
        .map_err(stringify_error)?
        .ban_members()
    {
        msg.channel_id
            .say(&ctx.http, "You don't have permission to unban")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    if msg.mentions.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Noone was mentioned, so noone was unbanned")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    let a = msg.guild(&ctx.cache).ok_or("Unknown Cache Error")?;
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
pub async fn ban(ctx: Context, msg: Message) -> Result<(), String> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member doesn't have the Admin permission
    if !msg
        .member(&ctx.http)
        .await
        .map_err(stringify_error)?
        .permissions(&ctx)
        .map_err(stringify_error)?
        .ban_members()
    {
        msg.channel_id
            .say(&ctx.http, "You don't have permission to ban")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    if msg.mentions.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Noone was mentioned, so noone was banned")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    if msg.mentions_me(&ctx).await.unwrap_or(true) {
        msg.channel_id
            .say(&ctx.http, "I won't ban myself")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    let a = msg.guild(&ctx.cache).ok_or("Unknown Cache Error")?;
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
pub async fn prefix(ctx: Context, msg: Message) -> Result<(), String> {
    let guild = if let Some(a) = msg.guild_id {
        a
    } else {
        // Ignore Private Messages
        return Ok(());
    };
    // if the member doesn't have the Admin permission
    if !msg
        .member(&ctx.http)
        .await
        .map_err(stringify_error)?
        .permissions(&ctx)
        .map_err(stringify_error)?
        .administrator()
    {
        msg.channel_id
            .say(
                &ctx.http,
                "Only administrators can change the server prefix",
            )
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    let new_prefix = msg.content.replace("`", "");
    if new_prefix.len() > 5 {
        msg.channel_id
            .say(&ctx.http, "Prefixes can't be longer than 5 symbols.")
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    prefix::set(guild, &ctx, &new_prefix).await;
    msg.channel_id
        .say(
            &ctx.http,
            format!("Successfully set prefix to `{}`", new_prefix),
        )
        .await
        .map_err(stringify_error)?;
    Ok(())
}
#[cfg(feature = "legacy_commands")]
pub async fn delete(ctx: Context, msg: Message) -> Result<(), String> {
    if msg.is_private() {
        return Ok(());
    }
    // if the member doesn't have the manage messages permission
    if !msg
        .member(&ctx.http)
        .await
        .map_err(stringify_error)?
        .permissions(&ctx.cache)
        .map_err(stringify_error)?
        .manage_messages()
    {
        msg.channel_id
            .say(&ctx.http, "You can't delete messages")
            .await
            .map_err(stringify_error)?;
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
            .await
            .map_err(stringify_error)?;
        sleep(Duration::from_secs(3)).await;
        error_msg.delete(&ctx.http).await.map_err(stringify_error)?;
        return Ok(());
    }
    let guild_channel = msg
        .channel(&ctx.http)
        .await
        .map_err(|x| format!("Error getting Channel: {x}"))?
        .guild()
        .ok_or("Unknown Cache Error getting Guild")?;
    let messages = guild_channel
        .messages(&ctx.http, |retriever| retriever.limit(amount + 1))
        .await
        .map_err(stringify_error)?;
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
            .await
            .map_err(stringify_error)?;
        sleep(Duration::from_secs(3)).await;
        error_msg.delete(&ctx.http).await.map_err(stringify_error)?;
    }
    Ok(())
}
