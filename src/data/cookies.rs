use serde_json::{self, Map, Value};
use std::fs;

pub struct Cookies<'a> {
    data: Map<String, Value>,
    path: &'a str,
}

impl<'a> Cookies<'a> {
    pub fn get(&self, user: u64) -> Option<u64> {
        self.data.get(&user.to_string())?.as_u64()
    }
    pub fn set(&mut self, user: u64, cookies: u64) {
        self.data.insert(user.to_string(), Value::from(cookies));
    }
    pub fn give(&mut self, user: u64, cookies: u64) {
        self.set(user, self.get(user).unwrap_or(0) + cookies);
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
