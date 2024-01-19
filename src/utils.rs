use crate::data::Data;

pub type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

pub const HELP_MESSAGE: &str = "\
Configure this bot by using the environment variables:
    DISCORD_TOKEN (required): The login token to use
    DISCORD_BOT_OWNERS (optional): Consider users with ids from this comma-seperated lists as owner
    DISCORD_HOME_SERVER (optional): Register Commands only on the server with this ID\
";
