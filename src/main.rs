#![warn(clippy::pedantic)]

mod commands;
mod data;
use anyhow::Context as _;
use data::{config::Config, Data};
use dotenv::dotenv;
use serenity::{
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    model::{
        application::{Command, Interaction},
        prelude::*,
    },
    prelude::*,
};
use std::env;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
struct Handler;

#[allow(clippy::no_effect_underscore_binding)]
#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let &Config { levels, cookies, .. } = ctx.data.read().await.get::<Config>().unwrap();
        if !(levels || cookies) {
            return;
        }

        // give xp and cookies to user
        let author_id = msg.author.id.get();
        if let Some((guild_id, data_lock)) = msg.guild_id.zip(ctx.data.read().await.get::<Data>()) {
            let mut data = data_lock.write().await;

            if cookies {
                data.cookies.give(author_id, fastrand::u64(0..2));
            }
            if levels {
                let xp = fastrand::u64(0..5);
                data.level.add_global_xp(author_id, xp);
                data.level.add_guild_xp(guild_id.get(), author_id, xp);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());
        let result = if let Some(guild) = ctx.data.read().await.get::<Config>().unwrap().home_server
        {
            Command::set_global_commands(&ctx.http, Vec::new()).await.ok();
            GuildId::from(guild).set_commands(&ctx.http, commands::commands()).await
        } else {
            Command::set_global_commands(&ctx.http, commands::commands()).await
        };
        if let Err(err) = result {
            eprintln!("Failed to register commands. More info:\n {err:#?}");
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else { return };
        let response = commands::respond_to(&ctx, &command).await;
        match response {
            Ok(msg) => command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content(msg),
                    ),
                )
                .await
                .unwrap_or_else(|why| eprintln!("An Error occurred: {why}")),
            Err(e) => {
                eprintln!("------------\n{e:?}\n------------\n{command:?}\n------------");
                command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(format!("An Error occurred: {e}\nIf you find a consistent way to cause this error, please report it to my support discord.")).ephemeral(true),
                        ),
                    )
                    .await
                    .unwrap_or_else(|why| eprintln!("An Error occurred: {why}"));
            }
        }
    }
}

impl Handler {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok(); // place variables from .env into this environment

    let token: String = env::var("DISCORD_TOKEN")
        .ok()
        .context("Put DISCORD_TOKEN=YourTokenHere into the file '.env' or the environment")?;
    let mut config = Config::from_file("config.toml").unwrap_or_default();

    // create client
    let mut client =
        Client::builder(&token, GatewayIntents::GUILD_MESSAGES).event_handler(Handler).await?;

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
