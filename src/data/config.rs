use std::env;

use poise::serenity_prelude::{GuildId, UserId};

#[derive(Debug, Default)]
pub struct Config {
    pub token: String,
    pub owners: Vec<UserId>,
    pub home_server: Option<GuildId>,
}
impl Config {
    pub fn from_env() -> Option<Self> {
        let Ok(token) = env::var("DISCORD_TOKEN") else {
            return None;
        };
        let owners = env::var("DISCORD_BOT_OWNERS")
            .unwrap_or_default()
            .split(',')
            .map(str::parse)
            .filter_map(Result::ok)
            .collect();
        let home_server = env::var("DISCORD_HOME_SERVER").unwrap_or_default().parse().ok();

        Some(Self { token, owners, home_server })
    }
}
