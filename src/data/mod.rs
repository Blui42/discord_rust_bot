pub mod prefix;
pub mod level;
pub mod cookies;

use serde::{Deserialize, Serialize};

pub struct Data {
    pub level: level::Level,
    pub cookies: cookies::Cookies,
}
impl Data {
    pub fn new() -> Self {
        Self{
            level: level::Level::new("level.json".to_string()),
            cookies: cookies::Cookies::new("cookies.json".to_string()),
        }
    }
    pub fn save(&self) {
        println!("Saving levels...");
        self.level.save();
        println!("Saving cookies...");
        self.cookies.save();
    }
}


#[derive(Deserialize, Serialize, Debug)]
pub struct LevelXP{
    pub level: u64,
    pub xp: u64,
}
impl LevelXP {
    pub fn new() -> Self{
        Self{level: 1, xp: 0}
    }
}
