use anyhow::{Context as CTX, Result};
use serenity::{
    model::{
        application::interaction::application_command::CommandDataOption, channel::Message,
        id::GuildId,
    },
    prelude::*,
};

#[cfg(feature = "legacy_commands")]
pub async fn ping(ctx: Context, msg: Message) -> Result<()> {
    msg.channel_id.say(&ctx.http, "pong!").await?;
    Ok(())
}

#[cfg(feature = "legacy_commands")]
pub async fn pic(ctx: Context, msg: Message) -> Result<()> {
    if msg.mentions.is_empty() {
        msg.channel_id.say(&ctx.http, msg.author.face()).await?;
        return Ok(());
    }
    for user in &msg.mentions {
        msg.channel_id.say(&ctx.http, user.face()).await?;
    }
    Ok(())
}
#[cfg(feature = "legacy_commands")]
pub async fn id(ctx: Context, msg: Message) -> Result<()> {
    for target in msg.content.trim().split_whitespace() {
        if !(target.starts_with('<') && target.ends_with('>')) {
            msg.channel_id
                .say(
                    &ctx.http,
                    "That's not a valid target. Mention a user, role, channel, etc",
                )
                .await?;
            continue;
        }
        let id = target.trim_matches(|c| !char::is_ascii_alphanumeric(&c));
        msg.channel_id.say(&ctx.http, id).await?;
    }
    Ok(())
}

pub async fn get_id_command(
    options: &[CommandDataOption],
    guild_id: Option<&GuildId>,
) -> Result<String> {
    dbg!(options);
    let target_type = options.get(0).context("get id target type")?;
    match target_type.name.as_str() {
        "user" | "channel" => Ok(options
            .get(0)
            .context("get first argument")?
            .options
            .get(0)
            .context("get seccond argument")?
            .value
            .as_ref()
            .context("get value of seccond argument")?
            .as_str()
            .context("get string representation of 2nd arg")?
            .to_string()),
        "server" => match guild_id {
            Some(a) => Ok(a.0.to_string()),
            None => Ok("This can only be used while on a server".to_string()),
        },
        _ => Ok("Something went wrong".to_string()),
    }
}
