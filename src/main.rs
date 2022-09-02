#![warn(clippy::pedantic)]

mod commands;
mod data;
use anyhow::Context as _;
use data::{config::Config, Data};
use dotenv::dotenv;
use serenity::{
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType::ChannelMessageWithSource},
        },
        prelude::*,
    },
    prelude::*,
};
use std::env;
use tokio::time::{sleep, Duration};

struct Handler;

#[allow(clippy::no_effect_underscore_binding)]
#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // ignore messages from bots
        if (!msg.author.bot) && (!msg.is_private()) {
            // give xp and cookies to user
            data::reward_user(&msg, &ctx).await;
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command = if let Interaction::ApplicationCommand(command) = interaction {
            command
        } else {
            return;
        };
        let options = command.data.options.as_slice();
        let response = match command.data.name.as_str() {
            "roll" => commands::fun::roll(options).await,
            "coin" => commands::fun::coin().await,
            "id" => commands::info::id(options, command.guild_id.as_ref()).await,
            "ttt" => commands::tic_tac_toe::command(options, &ctx, &command.user).await,
            "picture" => commands::info::picture(options).await,
            "delete" => commands::admin::delete(options, command.channel_id, &ctx).await,
            x => Err(anyhow::anyhow!("Unknown Command: {x}")),
        };
        match response {
            Ok(msg) => command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(msg))
                })
                .await
                .unwrap_or_else(|why| eprintln!("An Error occured: {why}")),
            Err(e) => {
                eprintln!("------------\n{e:?}\n------------\n{command:?}\n------------");
                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message
                                .content(format!("An Error occured: {e}\nIf you find a consistent way to cause this error, please report it to my support discord."))
                                .ephemeral(true)
                            })
                    })
                    .await.unwrap_or_else(|why| eprintln!("An Error occured: {why}"));
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());
        drop(Command::set_global_application_commands(&ctx.http, commands::commands).await);
        // drop(id::GuildId(792489181774479400).set_application_commands(&ctx.http, commands::commands).await);
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok(); // place variables from .env into this enviroment

    let token: String = env::var_os("DISCORD_TOKEN")
        .context("Put DISCORD_TOKEN=YourTokenHere into the .env file the enviroment")?
        .into_string()
        .ok()
        .context("DISCORD_TOKEN contained non-UTF8 characters")?;
    let config = Config::from_file("config.toml").unwrap_or_default();

    // Try to read Application ID from config.toml.
    // On failure, try to derive Application ID from bot token.
    let application_id = match config.application_id {
        0 => {
            serenity::utils::parse_token(&token)
                .context("Application ID was not given and could not be derived from token.")?
                .0
                 .0
        }
        a => a,
    };
    // create client
    let mut client: Client = Client::builder(
        &token,
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES,
    )
    .application_id(application_id)
    .event_handler(Handler)
    .await?;

    {
        let mut client_data = client.data.write().await;
        client_data.insert::<Data>(RwLock::new(Data::new()));
        client_data.insert::<Config>(config);

        #[cfg(feature = "tic_tac_toe")]
        client_data.insert::<commands::tic_tac_toe::Running>(RwLock::new(Vec::with_capacity(3)));
        #[cfg(feature = "tic_tac_toe")]
        client_data.insert::<commands::tic_tac_toe::Queue>(RwLock::new(Vec::with_capacity(3)));
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        println!("Shutting Down...");
        shard_manager.lock().await.shutdown_all().await;
        println!("Successfully Shut Down");
    });

    // automatically save every 10 minutes
    #[cfg(feature = "save_data")]
    {
        let client_data = client.data.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(600)).await;
                println!("Saving...");
                let res = client_data
                    .read()
                    .await
                    .get::<Data>()
                    .expect("Missing Data for saving!")
                    .read()
                    .await
                    .save()
                    .await;
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
