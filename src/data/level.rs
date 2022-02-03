#![allow(dead_code)]
#![cfg(feature = "xp")]
use serde::{Deserialize, Serialize};
use serde_json::{self, Map, Value};
use std::fs;

pub struct Level {
    data: Map<String, Value>,
    path: String,
}

impl Level {
    pub fn get(&self, user: &u64, guild: &u64) -> Option<LevelXP> {
        let a = self.data.get(&guild.to_string())?.get(user.to_string())?;
        serde_json::from_value(a.clone()).ok()?
    }
    pub fn set(&mut self, user: &u64, guild: &u64, to: &LevelXP) {
        if let Some(a) = self.data.get_mut(&guild.to_string()) {
            if let Some(b) = a.get_mut(user.to_string()) {
                if let Ok(to_as_value) = serde_json::to_value(to) {
                    *b = to_as_value;
                    return;
                }
            }
        }
        self.add_user(user, guild)
    }
    pub fn add_xp(&mut self, user: &u64, guild: &u64, xp: u64) {
        if let Some(mut current_level) = self.get(user, guild) {
            current_level.add_xp(xp);
            self.set(user, guild, &current_level);
            return;
        }
        self.add_user(user, guild)
    }
    fn add_user(&mut self, user: &u64, guild: &u64) {
        if let Some(a) = self.data.get_mut(&guild.to_string()) {
            if let Some(map) = a.as_object_mut() {
                map.insert(
                    user.to_string(),
                    serde_json::to_value(LevelXP::new()).unwrap(),
                );
            }
            return;
        }
        self.add_guild(guild)
    }
    fn add_guild(&mut self, guild: &u64) {
        self.data.insert(guild.to_string(), serde_json::json!({}));
    }
    pub fn new(path: String) -> Self {
        let file_contents = if let Ok(a) = fs::read_to_string(&path) {
            a
        } else {
            "{}".to_string()
        };
        let data: Map<String, Value> = serde_json::from_str(&file_contents).unwrap_or_default();
        Self { data, path }
    }

    pub fn save(&self) {
        if let Ok(file_info) = serde_json::to_string_pretty(&self.data) {
            if let Err(why) = fs::write(&self.path, file_info) {
                eprintln!("Error writing to file: {}", why);
            }
        }
    }
}
impl Drop for Level {
    fn drop(&mut self) {
        self.save();
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LevelXP {
    pub level: u64,
    pub xp: u64,
}
impl LevelXP {
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
