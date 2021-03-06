#![cfg(feature = "xp")]
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
        serde_json::from_value(a.clone()).ok()?
    }
    pub fn set(&mut self, user: u64, guild: u64, to: &XPCounter) {
        if let Some(a) = self.data.get_mut(&guild.to_string()) {
            if let Some(b) = a.get_mut(user.to_string()) {
                if let Ok(to_as_value) = serde_json::to_value(to) {
                    *b = to_as_value;
                    return;
                }
            }
        }
        self.add_user(user, guild);
    }
    pub fn add_xp(&mut self, user: u64, guild: u64, xp: u64) {
        if let Some(mut current_level) = self.get(user, guild) {
            current_level.add_xp(xp);
            self.set(user, guild, &current_level);
            return;
        }
        self.add_user(user, guild);
    }
    fn add_user(&mut self, user: u64, guild: u64) {
        if let Some(a) = self.data.get_mut(&guild.to_string()) {
            if let Some(map) = a.as_object_mut() {
                map.insert(
                    user.to_string(),
                    serde_json::to_value(XPCounter::new()).unwrap(),
                );
            }
            return;
        }
        self.add_guild(guild);
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
