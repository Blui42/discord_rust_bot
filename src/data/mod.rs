pub mod prefix;
pub mod level;
pub mod cookies;
pub mod config;

use rand::{Rng, SeedableRng, rngs::SmallRng};
use serde::{Deserialize, Serialize};
use serenity::{client::Context, model::channel::Message};

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

impl serenity::prelude::TypeMapKey for Data {
    type Value = Self;
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


// give the user cookies and xp
pub async fn reward_user(msg: &Message, ctx: &mut Context){
    let author_id = msg.author.id.0;
    if let Some(data) = ctx.data.write().await.get_mut::<Data>(){
        let mut rng = SmallRng::from_entropy();
        data.cookies.give(&author_id, rng.gen_range(0..2)); // cookies
        // xp
        let xp = rng.gen_range(0..5);
        data.level.add_xp(&author_id, &0, xp); // global xp
        if let Some(a) = msg.guild_id {
            data.level.add_xp(&author_id, &a.0, xp); // guild-specific xp
            return;
        }
    }
}