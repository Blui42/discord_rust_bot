pub mod config;
#[cfg(feature = "cookies")]
pub mod cookies;
#[cfg(feature = "xp")]
pub mod level;

use std::sync::Arc;

pub use cookies::Cookies;
pub use level::Level;
use tokio::sync::RwLock;

pub struct Data<'l, 'c> {
    #[cfg(feature = "xp")]
    pub level: Level<'l>,

    #[cfg(feature = "cookies")]
    pub cookies: Cookies<'c>,
}
impl Data<'static, 'static> {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "xp")]
            level: Level::new("level.json"),
            #[cfg(feature = "cookies")]
            cookies: Cookies::new("cookies.json"),
        }
    }
}
impl<'l, 'c> Data<'l, 'c> {
    pub async fn save(&self) -> Result<(), std::io::Error> {
        tokio::try_join!(
            #[cfg(feature = "xp")]
            self.level.save(),
            #[cfg(feature = "cookies")]
            self.cookies.save(),
        )?;

        Ok(())
    }
}

impl serenity::prelude::TypeMapKey for Data<'static, 'static> {
    type Value = Arc<RwLock<Self>>;
}
