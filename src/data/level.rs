#![allow(dead_code)]
use std::fs;
use serde_json;
use super::LevelXP;



pub struct LevelGuild{
    data: serde_json::Value,
    path: String,
}



impl LevelGuild{
    pub fn get(&self, user: &u64, guild: &u64) -> Option<LevelXP>{
        let a = self.data
            .get(guild.to_string())?
            .get(user.to_string())?;
        serde_json::from_value(a.clone()).ok()?
    }
    pub fn set(&mut self, user: &u64, guild: &u64, to: &LevelXP){
        if let Some(a) = self.data.get_mut(guild.to_string()){
            if let Some(b) = a.get_mut(user.to_string()){
                if let Ok(to_as_value) = serde_json::to_value(to){
                    *b = to_as_value;
                    return
                }
            }
        }
        self.add_user(user, guild)
    }
    pub fn add_xp(&mut self, user: &u64, guild: &u64, xp: u64) {
        if let Some(mut current_level) = self.get(user, guild){
            current_level.xp += xp;
            let xp_target = current_level.level * 10;
            if current_level.xp > xp_target {
                current_level.xp -= xp_target;
                current_level.level += 1;
            }
            self.set(user, guild, &current_level);
            return;
        }
        self.add_user(user, guild)
    }
    fn add_user(&mut self, user: &u64, guild: &u64){
        if let Some(a) = self.data.get_mut(guild.to_string()){
            if let Some(map) = a.as_object_mut(){
                map.insert(user.to_string(), serde_json::to_value(LevelXP::new()).unwrap());
            }
            return;
        }
        self.add_guild(guild)
    }
    fn add_guild(&mut self, guild: &u64){
        if let Some(map) = self.data.as_object_mut(){
            map.insert(guild.to_string(), serde_json::json!({}));
        }
    }
    pub fn new(path: String) -> Self{
        let file_contents = if let Ok(a) = fs::read_to_string(&path) {a} else {"{}".to_string()};
        let data: serde_json::Value = serde_json::from_str(&file_contents).unwrap_or_default();
        Self{data, path}
    }
    
    pub fn save(&self){
        if let Ok(file_info) = serde_json::to_string_pretty(&self.data){
            if let Err(why) = fs::write(&self.path, file_info){
                eprintln!("Error writing to file: {}", why);
            }
        }
    }
}
impl Drop for LevelGuild{
    fn drop(&mut self) { 
        self.save();
    }
}


