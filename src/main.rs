mod commands;
mod data;
use commands::*;
use data::{config::Config, prefix::Prefix, Data};
use dotenv::dotenv;
use serenity::{
    client::bridge::gateway::GatewayIntents,
    model::{
        channel::Message,
        gateway::Ready,
        interactions::{application_command::ApplicationCommand, Interaction},
        prelude::*,
    },
    prelude::*,
};
use std::env;
use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // ignore messages from bots
        if msg.author.bot {
            return;
        }

        #[cfg(not(feature = "respond_dm"))]
        if msg.is_private() {
            return;
        }

        #[cfg(not(feature = "respond_server"))]
        if !msg.is_private() {
            return;
        }

        // get the prefix for the current guild
        #[cfg(feature = "custom_prefix")]
        let prefix = if let Some(guild_id) = msg.guild_id {
            match data::prefix::get_prefix(&msg, &ctx).await {
                Some(a) => a,
                None => {
                    data::prefix::set_prefix(&guild_id, &ctx, "!").await;
                    "!".to_string()
                }
            }
        } else {
            "!".to_string()
        };
        #[cfg(not(feature = "custom_prefix"))]
        let prefix: String = "!";
        // gives xp and cookies to user
        data::reward_user(&msg, &ctx).await;

        #[cfg(feature = "legacy_commands")]
        commands::parse_command(&prefix, msg, ctx).await;
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command = if let Interaction::ApplicationCommand(command) = interaction {
            command
        } else {
            return;
        };
        let response = match command.data.name.as_str() {
            "roll" => fun::roll_command(command.data.options.as_slice()).await,
            "coin" => fun::coin_command().await,
            "id" => info::get_id_command(command.data.options.as_slice()).await,
            "ttt" => {
                tic_tac_toe::command(command.data.options.as_slice(), &ctx, &command.user).await
            }
            _ => None,
        };
        if let Some(a) = response {
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(interactions::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(a))
                })
                .await
            {
                eprintln!("An Error occured: {}", why)
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(interactions::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("An Error occured."))
                })
                .await
            {
                eprintln!("An Error occured: {}", why)
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());
        let _ = ApplicationCommand::set_global_application_commands(&ctx.http, commands).await;
        // let _ = id::GuildId(792489181774479400).set_application_commands(&ctx.http, commands).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // place variables from .env into this enviroment

    let token = env::var("DISCORD_TOKEN") // load DISCORD_TOKEN from enviroment
        .expect("Put DISCORD_TOKEN=YourTokenHere into the .env file or add a DISCORD_TOKEN variable to the enviroment"); // panic if not present

    let config = data::config::Config::from_file("config.toml").unwrap_or_default();

    println!("{:#?}", &config);

    // Try to read Application ID from config.toml.
    // On failure, try to derive Application ID from bot token.
    let application_id = match config.application_id {
        0 => {
            serenity::client::parse_token(&token)
                .expect("Application ID was not given and could not be derived from token.")
                .bot_user_id
                .0
        }
        a => a,
    };
    // create client
    let mut client: Client = Client::builder(&token)
        .intents(
            GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::GUILD_BANS,
        )
        .application_id(application_id)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut client_data = client.data.write().await;
        let data = Data::new();
        client_data.insert::<Data>(RwLock::new(data));
        client_data.insert::<Config>(config);

        #[cfg(feature = "custom_prefix")]
        client_data.insert::<Prefix>(RwLock::new(Prefix::new("prefix.json".to_string())));

        #[cfg(feature = "tic_tac_toe")]
        client_data.insert::<tic_tac_toe::TicTacToeRunning>(RwLock::new(Vec::with_capacity(3)));
        #[cfg(feature = "tic_tac_toe")]
        client_data.insert::<tic_tac_toe::TicTacToeQueue>(RwLock::new(Vec::with_capacity(3)));
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
            sleep(Duration::from_secs(600)).await;
            println!("Preparing to save...");
            tokio::join!(
                async {
                    if let Some(readable_data) = client_data.read().await.get::<Data>() {
                        readable_data.read().await.save();
                    } else {
                        eprintln!("Data to save was not accessible");
                    }
                },
                #[cfg(feature = "custom_prefix")]
                async {
                    if let Some(prefix) = client_data.read().await.get::<Prefix>() {
                        prefix.read().await.save();
                    } else {
                        eprintln!("Data to save was not accessible");
                    }
                }
            )
        });
    }

    if let Err(why) = client.start_autosharded().await {
        println!("Client error: {:?}", why);
    };
}
