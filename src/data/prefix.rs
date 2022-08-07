use serde_json::{self, Map, Value};
use serenity::{
    client::Context,
    model::{channel::Message, id::GuildId},
};
use std::fs;
use tokio::sync::RwLock;

pub struct Prefix<'a> {
    prefix: Map<String, Value>,
    path: &'a str,
}

impl<'a> Prefix<'a> {
    pub fn get(&self, guild: u64) -> Option<String> {
        self.prefix
            .get(&guild.to_string())?
            .as_str()
            .map(ToString::to_string)
    }
    pub fn set(&mut self, guild: u64, prefix: impl Into<String>) {
        self.prefix.insert(guild.to_string(), prefix.into().into());
    }
    pub fn new(path: &'a str) -> Self {
        let file = fs::read_to_string(&path);
        let file_contents: &str = file.as_ref().map_or("{}", |x| x);
        let prefix: Map<_, _> = serde_json::from_str(file_contents).unwrap_or_default();
        Self { prefix, path }
    }
    pub async fn save(&self) -> Result<(), std::io::Error> {
        if let Ok(file_info) = serde_json::to_string_pretty(&self.prefix) {
            tokio::fs::write(self.path, file_info).await?;
        }
        Ok(())
    }
}

impl<'a> Drop for Prefix<'a> {
    fn drop(&mut self) {
        if let Ok(file_content) = serde_json::to_string_pretty(&self.prefix) {
            if let Err(why) = fs::write(&self.path, file_content) {
                eprintln!("Error writing to file: {why}");
            }
        }
    }
}

impl serenity::prelude::TypeMapKey for Prefix<'static> {
    type Value = RwLock<Self>;
}

/// Tries to get the prefix for the guild the user is in.
/// If this returns None, a default value of "!" should be used.
pub async fn get(msg: &Message, ctx: &Context) -> Option<String> {
    // early return for private messages
    let guild_id = msg.guild_id?.0;

    ctx.data
        .read()
        .await
        .get::<Prefix>()?
        .read()
        .await
        .get(guild_id)
}
pub async fn set(guild: GuildId, ctx: &Context, new_prefix: &str) {
    // get mutable prefix variable
    if let Some(prefix_lock) = ctx.data.read().await.get::<crate::Prefix>() {
        let mut prefix = prefix_lock.write().await;
        prefix.set(guild.0, new_prefix);
    }
}
