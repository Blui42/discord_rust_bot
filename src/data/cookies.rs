#![cfg(feature = "cookies")]

use serde_json::{self, Map, Value};
use std::fs;

pub struct Cookies {
    data: Map<String, Value>,
    path: String,
}

impl Cookies {
    pub fn get(&self, user: u64) -> Option<u64> {
        let a = self.data.get(&user.to_string())?;
        a.as_u64()
    }
    pub fn set(&mut self, user: u64, cookies: u64) {
        if let Some(b) = self.data.get_mut(&user.to_string()) {
            *b = Value::from(cookies);
            return;
        }
        self.add_user(user);
    }
    pub fn give(&mut self, user: u64, cookies: u64) {
        if let Some(current_cookies) = self.get(user) {
            self.set(user, current_cookies + cookies);
            return;
        }
        self.add_user(user);
    }
    fn add_user(&mut self, user: u64) {
        self.data.insert(user.to_string(), Value::from(0_u64));
    }

    pub fn new(path: String) -> Self {
        let file_contents = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
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
impl Drop for Cookies {
    fn drop(&mut self) {
        self.save();
    }
}
