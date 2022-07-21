#![cfg(feature = "cookies")]

use serde_json::{self, Map, Value};
use std::fs;

pub struct Cookies<'a> {
    data: Map<String, Value>,
    path: &'a str,
}

impl<'a> Cookies<'a> {
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

    pub fn new(path: &'a str) -> Self {
        let file = fs::read_to_string(path);
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
impl<'a> Drop for Cookies<'a> {
    fn drop(&mut self) {
        if let Ok(file_content) = serde_json::to_string_pretty(&self.data) {
            if let Err(why) = fs::write(&self.path, file_content) {
                eprintln!("Error writing to file: {why}");
            }
        }
    }
}
