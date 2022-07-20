#![warn(clippy::pedantic)]

mod commands;
mod data;
use data::{config::Config, prefix::Prefix, Data};
use dotenv::dotenv;
use serenity::{
    model::{
        application::{
            command::Command,
            interaction::Interaction,
        },
        prelude::*,
    },
    prelude::*,
};
use std::{borrow::Cow, env, io};
use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

struct Handler;

#[allow(clippy::no_effect_underscore_binding)]
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
        let prefix = match data::prefix::get(&msg, &ctx).await {
            Some(a) => Cow::Owned(a),
            None => Cow::Borrowed("!"),
        };
        #[cfg(not(feature = "custom_prefix"))]
        let prefix = "!";
        // gives xp and cookies to user
        data::reward_user(&msg, &ctx).await;

        #[cfg(feature = "legacy_commands")]
        commands::parse(&prefix, msg, ctx).await;
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command = if let Interaction::ApplicationCommand(command) = interaction {
            command
        } else {
            return;
        };
        let options = command.data.options.as_slice();
        let response = match command.data.name.as_str() {
            "roll" => commands::fun::roll_command(options).await,
            "coin" => commands::fun::coin_command().await,
            "id" => commands::info::get_id_command(options, command.guild_id.as_ref()).await,
            "ttt" => commands::tic_tac_toe::command(options, &ctx, &command.user).await,
            x => Err(anyhow::anyhow!("Unknown Command: {x}")),
        };
        match response {
            Ok(msg) => command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(interaction::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(msg))
                })
                .await
                .unwrap_or_else(|why| eprintln!("An Error occured: {why}")),
            Err(e) => {
                eprintln!("------------\n{e:?}\n------------");
                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(interaction::InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message
                                .content(format!("An Error occured: {e}\nIf you find a consistent way to cause this error, please report it to my support discord."))
                                .flags(interaction::MessageFlags::EPHEMERAL)
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
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, 
        "Put DISCORD_TOKEN=YourTokenHere into the .env file or add a DISCORD_TOKEN variable to the enviroment"))?.into_string()
        .map_err(|_|io::Error::new(io::ErrorKind::Other, "DISCORD_TOKEN contained non-UTF8 characters"))?;
    let config = data::config::Config::from_file("config.toml").unwrap_or_default();

    // Try to read Application ID from config.toml.
    // On failure, try to derive Application ID from bot token.
    let application_id = match config.application_id {
        0 => {
            serenity::utils::parse_token(&token)
                .expect("Application ID was not given and could not be derived from token.")
                .0
                 .0
        }
        a => a,
    };
    // create client
    let mut client: Client = Client::builder(
        &token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::GUILD_BANS,
    )
    .application_id(application_id)
    .event_handler(Handler)
    .await?;

    {
        let mut client_data = client.data.write().await;
        let data = Data::new();
        client_data.insert::<Data>(RwLock::new(data));
        client_data.insert::<Config>(config);

        #[cfg(feature = "custom_prefix")]
        client_data.insert::<Prefix>(RwLock::new(Prefix::new("prefix.json".to_string())));

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
                println!("Preparing to save...");
                let res: Result<_, ()> = tokio::try_join!(
                    async {
                        client_data
                            .read()
                            .await
                            .get::<Data>()
                            .ok_or(())?
                            .read()
                            .await
                            .save();
                        Ok(())
                    },
                    #[cfg(feature = "custom_prefix")]
                    async {
                        client_data
                            .read()
                            .await
                            .get::<Prefix>()
                            .ok_or(())?
                            .read()
                            .await
                            .save();
                        Ok(())
                    }
                );
                if res.is_err() {
                    eprintln!("Something went wrong trying to save. Disabled saving.");
                    return;
                }
            }
        });
    }

    client.start_autosharded().await?;
    Ok(())
}
