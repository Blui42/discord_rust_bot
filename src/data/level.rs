use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;

pub struct Level<'a> {
    data: Map<String, Value>,
    path: &'a str,
}

impl<'a> Level<'a> {
    pub fn get(&self, user: u64, guild: u64) -> Option<XPCounter> {
        let a = self.data.get(&guild.to_string())?.get(user.to_string())?;
        serde_json::from_value(a.clone()).ok()
    }
    pub fn set(&mut self, user: u64, guild: u64, to: &XPCounter) {
        if let Some(a) = self.data.get_mut(&guild.to_string()).and_then(Value::as_object_mut) {
            a.insert(user.to_string(), serde_json::to_value(to).unwrap());
        }
        self.add_guild(guild);
    }
    pub fn add_guild_xp(&mut self, user: u64, guild: u64, xp: u64) {
        let mut current_level = self.get(user, guild).unwrap_or_default();
        current_level.add_xp(xp);
        self.set(user, guild, &current_level);
    }
    pub fn add_global_xp(&mut self, user: u64, xp: u64) {
        self.add_guild_xp(user, 0, xp);
    }

    fn add_guild(&mut self, guild: u64) {
        self.data.insert(guild.to_string(), serde_json::json!({}));
    }
    pub fn new(path: &'a str) -> Self {
        let file = fs::read_to_string(&path);
        let file_contents: &str = file.as_ref().map_or("{}", |x| x);
        let data: Map<String, Value> = serde_json::from_str(file_contents).unwrap_or_default();
        Self { data, path }
    }

    pub async fn save(&self) -> Result<(), std::io::Error> {
        if let Ok(file_info) = serde_json::to_string_pretty(&self.data) {
            tokio::fs::write(&self.path, file_info).await?;
        }
        Ok(())
    }
}
impl<'a> Drop for Level<'a> {
    fn drop(&mut self) {
        if let Ok(file_content) = serde_json::to_string_pretty(&self.data) {
            if let Err(why) = fs::write(&self.path, file_content) {
                eprintln!("Error writing to file: {why}");
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct XPCounter {
    pub level: u64,
    pub xp: u64,
}
impl XPCounter {
    pub fn new() -> Self {
        Self { level: 1, xp: 0 }
    }

    pub fn add_xp(&mut self, extra_xp: u64) {
        self.xp += extra_xp;
        let xp_target = self.level * 10;
        while self.xp > xp_target {
            self.xp -= xp_target;
            self.level += 1;
        }
    }
}

impl Default for XPCounter {
    fn default() -> Self {
        Self::new()
    }
}
