use std::borrow::Cow;

use anyhow::{bail, Context as _, Result};
use serenity::model::{
    application::{CommandDataOption, CommandDataOptionValue, ResolvedOption, ResolvedValue},
    id::GuildId,
};

pub async fn id<'a>(
    options: &'a [CommandDataOption],
    guild_id: Option<&GuildId>,
) -> Result<Cow<'a, str>> {
    let subcommand = options.get(0).context("Missing Subcommand")?;
    let CommandDataOptionValue::SubCommand(subcommand_options) = &subcommand.value else {
        bail!("Missing Subcommand Arguments")
    };
    match subcommand.name.as_str() {
        "user" | "channel" => subcommand_options
            .get(0)
            .and_then(|arg| arg.value.as_str())
            .map(Into::into)
            .ok_or_else(|| anyhow::anyhow!("Missing Argument (API out of sync?)")),
        "server" => guild_id.map_or(Ok("This can only be used on servers".into()), |id| {
            Ok(id.get().to_string().into())
        }),
        _ => Ok("Something went wrong".into()),
    }
}
pub async fn picture(options: &[ResolvedOption<'_>]) -> Result<Cow<'static, str>> {
    let target = options.get(0).map(|val| &val.value);
    match target {
        Some(ResolvedValue::User(target, _)) => Ok(target.face().into()),
        _ => anyhow::bail!("Missing Arguments for getting profile picture"),
    }
}
