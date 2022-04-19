use crate::stringify_error;
use serenity::{
    model::{
        channel::Message, id::GuildId,
        interactions::application_command::ApplicationCommandInteractionDataOption,
    },
    prelude::*,
};

#[cfg(feature = "legacy_commands")]
pub async fn ping(ctx: Context, msg: Message) -> Result<(), String> {
    msg.channel_id
        .say(&ctx.http, "pong!")
        .await
        .map_err(stringify_error)?;
    Ok(())
}

#[cfg(feature = "legacy_commands")]
pub async fn pic(ctx: Context, msg: Message) -> Result<(), String> {
    if msg.mentions.is_empty() {
        msg.channel_id
            .say(&ctx.http, msg.author.face())
            .await
            .map_err(stringify_error)?;
        return Ok(());
    }
    for i in msg.mentions.iter() {
        msg.channel_id
            .say(&ctx.http, i.face())
            .await
            .map_err(stringify_error)?;
    }
    Ok(())
}
#[cfg(feature = "legacy_commands")]
pub async fn id(ctx: Context, msg: Message) -> Result<(), String> {
    for target in msg.content.split_whitespace() {
        if !(target.starts_with('<') && target.ends_with('>')) {
            msg.channel_id
                .say(
                    &ctx.http,
                    "That's not a valid target. Mention a user, role, channel, etc",
                )
                .await
                .map_err(stringify_error)?;
            continue;
        }
        let mut id = target.to_string();
        for i in &["<", ">", "!", "#", "@", "&"] {
            id = id.replace(i, "");
        }
        msg.channel_id
            .say(&ctx.http, id)
            .await
            .map_err(stringify_error)?;
    }
    Ok(())
}

pub async fn get_id_command(
    options: &[ApplicationCommandInteractionDataOption],
    guild_id: Option<&GuildId>,
) -> Option<String> {
    dbg!(options);
    let target_type = options.get(0)?;
    match target_type.name.as_str() {
        "user" | "channel" => Some(
            options
                .get(0)?
                .options
                .get(0)?
                .value
                .as_ref()?
                .as_str()?
                .to_string(),
        ),
        "server" => match guild_id {
            Some(a) => Some(a.0.to_string()),
            None => Some("This can only be used while on a server".to_string()),
        },
        _ => Some("Something went wrong".to_string()),
    }
}
