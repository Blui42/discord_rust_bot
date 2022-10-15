#![allow(dead_code)]
use serde::Deserialize;
use std::{fs, num::NonZeroU64};
extern crate toml;

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub application_id: Option<NonZeroU64>,
    #[serde(default)]
    pub home_server: Option<NonZeroU64>,
    #[serde(default)]
    pub owners: Vec<u64>,
    #[serde(default)]
    pub levels: bool,
    #[serde(default)]
    pub cookies: bool,
}
impl Config {
    pub fn from_file(path: &str) -> Option<Self> {
        let file_content = fs::read(path).ok()?;
        return toml::from_slice::<Self>(file_content.as_slice()).ok();
    }
}

impl serenity::prelude::TypeMapKey for Config {
    type Value = Self;
}

impl Default for Config {
    fn default() -> Self {
        Self {
            application_id: None,
            home_server: None,
            owners: Vec::new(),
            cookies: true,
            levels: true,
        }
    }
}
