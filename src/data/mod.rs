pub mod config;
pub mod cookies;
pub mod level;

use std::sync::Arc;

pub use cookies::Cookies;
pub use level::Level;
use tokio::sync::RwLock;

pub struct Data<'l, 'c> {
    pub level: Level<'l>,
    pub cookies: Cookies<'c>,
}
impl Default for Data<'static, 'static> {
    fn default() -> Self {
        Self { level: Level::new("level.json"), cookies: Cookies::new("cookies.json") }
    }
}
impl<'l, 'c> Data<'l, 'c> {
    pub async fn save(&self) -> Result<(), std::io::Error> {
        tokio::try_join!(self.level.save(), self.cookies.save(),)?;

        Ok(())
    }
}

impl serenity::prelude::TypeMapKey for Data<'static, 'static> {
    type Value = Arc<RwLock<Self>>;
}
