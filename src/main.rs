mod commands;
mod data;
mod utils;

use std::{collections::HashSet, iter::FromIterator, sync::Arc};

use anyhow::Context as _;
use data::{config::Config, Data};
use poise::{
    builtins,
    serenity_prelude::{Client, GatewayIntents},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok(); // place variables from .env into this environment

    let Config { token, owners, home_server } = Config::from_env().context(utils::HELP_MESSAGE)?;

    let framework = poise::Framework::new(
        poise::FrameworkOptions {
            skip_checks_for_owners: true,
            commands: commands::commands(),
            owners: HashSet::from_iter(owners),
            initialize_owners: true,
            ..Default::default()
        },
        move |ctx, ready, fw| {
            Box::pin(async move {
                println!("Connected as {}", ready.user.tag());
                if let Some(guild_id) = home_server {
                    builtins::register_in_guild(ctx, &fw.options().commands, guild_id).await?;
                } else {
                    builtins::register_globally(ctx, &fw.options().commands).await?;
                }
                Ok(Data::default())
            })
        },
    );
    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents).framework(framework).await?;

    let shard_manager = Arc::clone(&client.shard_manager);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        println!("Shutting Down...");
        shard_manager.shutdown_all().await;
    });

    client.start_autosharded().await?;
    Ok(())
}
