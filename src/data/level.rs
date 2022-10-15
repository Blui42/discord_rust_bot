use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub struct Level<'a> {
    data: HashMap<String, HashMap<String, XPCounter>>,
    path: &'a str,
}

impl<'a> Level<'a> {
    // Might get relevant later in development
    #[allow(dead_code)]
    pub fn get(&self, user: u64, guild: u64) -> Option<XPCounter> {
        self.data.get(&guild.to_string())?.get(&user.to_string()).cloned()
    }
    #[allow(dead_code)]
    pub fn set(&mut self, user: u64, guild: u64, to: XPCounter) {
        self.data
            .entry(guild.to_string())
            .or_insert_with(HashMap::new)
            .insert(user.to_string(), to);
    }
    pub fn add_guild_xp(&mut self, user: u64, guild: u64, xp: u64) {
        self.data
            .entry(guild.to_string())
            .or_insert_with(HashMap::new)
            .entry(user.to_string())
            .and_modify(|old_xp| old_xp.add_xp(xp))
            .or_insert_with(|| XPCounter::new_with_xp(xp));
    }
    pub fn add_global_xp(&mut self, user: u64, xp: u64) {
        self.add_guild_xp(user, 0, xp);
    }

    pub fn new(path: &'a str) -> Self {
        let file = fs::read_to_string(&path);
        let file_contents: &str = file.as_ref().map_or("{}", |x| x);
        let data: HashMap<_, _> = serde_json::from_str(file_contents).unwrap_or_default();
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct XPCounter {
    pub level: u64,
    pub xp: u64,
}
impl XPCounter {
    #[inline]
    pub const fn new() -> Self {
        Self { level: 1, xp: 0 }
    }

    pub fn new_with_xp(xp: u64) -> Self {
        let mut val = Self { level: 1, xp };
        val.level_up();
        val
    }

    pub fn add_xp(&mut self, extra_xp: u64) {
        self.xp += extra_xp;
        self.level_up();
    }

    pub fn level_up(&mut self) {
        let mut xp_target = self.level * 10;
        while self.xp > xp_target {
            self.xp -= xp_target;
            self.level += 1;
            xp_target += 10;
        }
    }
}

impl Default for XPCounter {
    fn default() -> Self {
        Self::new()
    }
}
