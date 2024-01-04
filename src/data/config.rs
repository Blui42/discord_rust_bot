#![allow(dead_code)]
use std::{fs, num::NonZeroU64};

use serde::Deserialize;

#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize, Debug)]
pub struct Config {
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
        let file_content = fs::read_to_string(path).ok()?;
        toml::from_str::<Self>(&file_content).ok()
    }
}

impl serenity::prelude::TypeMapKey for Config {
    type Value = Self;
}

impl Default for Config {
    fn default() -> Self {
        Self { home_server: None, owners: Vec::new(), cookies: true, levels: true }
    }
}
