use std::borrow::Cow;

use anyhow::{Context as _, Result};
use serenity::model::{
    application::interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    id::GuildId,
};

pub async fn id<'a>(
    options: &'a [CommandDataOption],
    guild_id: Option<&GuildId>,
) -> Result<Cow<'a, str>> {
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
            .into()),
        "server" => match guild_id {
            Some(a) => Ok(a.0.to_string().into()),
            None => Ok("This can only be used while on a server".into()),
        },
        _ => Ok("Something went wrong".into()),
    }
}
pub async fn picture<'a>(options: &'a [CommandDataOption]) -> Result<Cow<'static, str>> {
    let target = options.get(0).and_then(|x| x.resolved.as_ref());
    match target {
        Some(CommandDataOptionValue::User(target, _)) => Ok(target.face().into()),
        _ => anyhow::bail!("Missing Arguments for getting profile picture"),
    }
}
