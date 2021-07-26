use std::fs;
use serde_json;


pub struct Prefix{
    prefix: serde_json::Value,
    path: String,
}


impl Prefix{
    pub fn get(&self, guild: u64) -> Option<String>{
        if let Some(a) = self.prefix.get(guild.to_string())?.as_str(){  // check if the guild has a prefix
            return Some(a.to_string());  // if so, return it
        }
        return None  // otherwise, return None
    }
    pub fn set(&mut self, guild: u64, prefix:  &str){
        if let Some(a) = self.prefix.get_mut(guild.to_string()){
            *a = serde_json::json!(prefix);
            return;
        }
        self.add_prefix(guild, prefix)
    }
    fn add_prefix(&mut self, guild: u64, prefix:  &str){
        if let Some(map) = self.prefix.as_object_mut(){
            map.insert(guild.to_string(), serde_json::Value::String(prefix.to_string()));
        }
    }
    pub fn new(path: String) -> Self{
        let file_contents = if let Ok(a) = fs::read_to_string(&path) {a} else {"{}".to_string()};
        let prefix: serde_json::Value = serde_json::from_str(&file_contents).unwrap_or_default();
        return Self{path, prefix};
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

