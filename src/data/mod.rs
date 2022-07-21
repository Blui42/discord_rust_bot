pub mod config;
#[cfg(feature = "cookies")]
pub mod cookies;
#[cfg(feature = "xp")]
pub mod level;
pub mod prefix;

use rand::{thread_rng, Rng};
use serenity::{client::Context, model::channel::Message};
use tokio::sync::RwLock;

pub struct Data<'l, 'c> {
    #[cfg(feature = "xp")]
    pub level: level::Level<'l>,

    #[cfg(feature = "cookies")]
    pub cookies: cookies::Cookies<'c>,
}
impl Data<'static, 'static> {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "xp")]
            level: level::Level::new("level.json"),
            #[cfg(feature = "cookies")]
            cookies: cookies::Cookies::new("cookies.json"),
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
    type Value = RwLock<Self>;
}

// give the user cookies and xp
pub async fn reward_user(msg: &Message, ctx: &Context) {
    let author_id = msg.author.id.0;
    if let Some(data_lock) = ctx.data.read().await.get::<Data>() {
        let mut data = data_lock.write().await;
        let mut rng = thread_rng();

        #[cfg(feature = "cookies")]
        data.cookies.give(author_id, rng.gen_range(0..2));

        #[cfg(feature = "xp")]
        {
            let xp = rng.gen_range(0..5);
            data.level.add_xp(author_id, 0, xp); // global xp
            if let Some(a) = msg.guild_id {
                data.level.add_xp(author_id, a.0, xp); // guild-specific xp
            }
        }
    }
}
