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
        "user" | "channel" => subcommand
            .options
            .get(0)
            .and_then(|arg| arg.value.as_ref())
            .and_then(serde_json::Value::as_str)
            .map(Into::into)
            .ok_or_else(|| anyhow::anyhow!("Missing Argument (API out of sync?)")),
        "server" => guild_id.map_or(Ok("This can only be used on servers".into()), |id| {
            Ok(id.0.to_string().into())
        }),
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
