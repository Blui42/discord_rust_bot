#![allow(dead_code)]
use std::fs;
use serde::Deserialize;
extern crate toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub application_id: Option<u64>,
    pub home_server: Option<u64>,
    pub home_channel: Option<u64>,
    pub owners: Box<[u64]>,
    pub legacy_commands: bool,
    pub respond_dm: bool,
    pub respond_server: bool,
    pub commands_home_only: bool,
}
impl Config {
    pub fn from_file(path: &str) -> Option<Self> {
        let file_content = fs::read(path).ok()?;
        return toml::from_slice::<Self>(file_content.as_slice()).ok();
    }
}
impl Default for Config {
    fn default() -> Self {
        Self { 
            application_id: None, 
            home_server: None, 
            home_channel: None, 
            owners: Box::new([]), 
            legacy_commands: true, 
            respond_dm: true, 
            respond_server: true, 
            commands_home_only: false 
        }
    }
}