#![warn(clippy::pedantic)]

mod commands;
mod data;
mod handler;

use anyhow::Context as _;
use data::{config::Config, Data};
use handler::Handler;
use serenity::{client::Client, model::gateway::GatewayIntents};
use std::env;
use std::sync::Arc;
use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok(); // place variables from .env into this environment

    let token = env::var("DISCORD_TOKEN")
        .context("Put DISCORD_TOKEN=YourTokenHere into the file '.env' or the environment")?;
    let mut config = Config::from_file("config.toml").unwrap_or_default();

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents).event_handler(Handler).await?;

    if let Ok(info) = client.http.get_current_application_info().await {
        if let Some(team) = info.team {
            config.owners.extend(team.members.iter().map(|x| x.user.id.get()));
        }
        if let Some(owner) = info.owner {
            config.owners.push(owner.id.get());
        }
    }

    {
        use commands::{rock_paper_scissors, tic_tac_toe};

        let mut client_data = client.data.write().await;
        client_data.insert::<Data>(Arc::default());
        client_data.insert::<Config>(config);

        client_data.insert::<rock_paper_scissors::Queue>(Arc::default());

        client_data.insert::<tic_tac_toe::Running>(RwLock::default());
        client_data.insert::<tic_tac_toe::Queue>(RwLock::default());
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        println!("Shutting Down...");
        shard_manager.shutdown_all().await;
        println!("Successfully Shut Down");
    });

    // automatically save every 10 minutes
    if let Some(client_data) = client.data.read().await.get::<Data>().cloned() {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(600)).await;
                println!("Saving...");
                let res = client_data.read().await.save().await;
                if let Err(why) = res {
                    eprintln!("Error trying to save. Disabled saving. \nMore info: {why}");
                    return;
                }
            }
        });
    }

    client.start_autosharded().await?;
    Ok(())
}
