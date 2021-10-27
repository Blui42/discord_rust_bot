use std::fs;
use serde_json::{self, Map, Value};
use serenity::{client::Context, model::channel::Message};


pub struct Prefix{
    prefix: Map<String, Value>,
    path: String,
}


impl Prefix{
    pub fn get(&self, guild: u64) -> Option<String>{
        if let Some(a) = self.prefix.get(&guild.to_string())?.as_str(){  // check if the guild has a prefix
            return Some(a.to_string());  // if so, return it
        }
        None  // otherwise, return None
    }
    pub fn set(&mut self, guild: u64, prefix:  &str){
        if let Some(a) = self.prefix.get_mut(&guild.to_string()){
            *a = serde_json::json!(prefix);
            return;
        }
        self.add_prefix(guild, prefix)
    }
    fn add_prefix(&mut self, guild: u64, prefix:  &str){
        self.prefix.insert(guild.to_string(), Value::String(prefix.to_string()));
    }
    pub fn new(path: String) -> Self{
        let file_contents = if let Ok(a) = fs::read_to_string(&path) {a} else {"{}".to_string()};
        let prefix: serde_json::Map<String, Value> = serde_json::from_str(&file_contents).unwrap_or_default();
        Self{path, prefix}
    }
    pub fn save(&self){
        if let Ok(file_info) = serde_json::to_string_pretty(&self.prefix){
            if let Err(why) = fs::write(&self.path, file_info){
                eprintln!("Error writing to file: {}", why);
            }
        }
    }
}
impl Drop for Prefix{
    fn drop(&mut self) { 
        self.save();
    }
}

pub async fn get_prefix(msg: &Message, ctx: &Context) -> Option<String>{
    // return "!" for PMs
    if msg.is_private(){return Some("!".to_string())}
    // get immutable reference to prefix variable
    return ctx.data
        .read().await
        .get::<crate::Prefix>()?
        .get(msg.guild_id?.0);
}
pub async fn set_prefix(msg: &Message, ctx: &mut Context, new_prefix: &str){
    // get mutable prefix variable
    if let Some(prefix) = ctx.data.write().await.get_mut::<crate::Prefix>(){
        if let Some(a) = msg.guild_id {
            prefix.set(a.0, new_prefix);
            return;
        }
    }
}