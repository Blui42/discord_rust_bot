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
    let subcommand = options.get(0).context("Missing Subcommand")?;
    match subcommand.name.as_str() {
        "user" | "channel" => Ok(subcommand
            .options
            .get(0)
            .context("Missing Argument (API out of sync?)")?
            .value
            .as_ref()
            .context("Missing Argument Value (API out of sync?)")?
            .as_str()
            .context("Argument was not a String! (I may need an update?)")?
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
