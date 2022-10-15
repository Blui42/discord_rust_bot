#![warn(clippy::pedantic)]

mod commands;
mod data;
use anyhow::Context as _;
use data::{config::Config, Data};
use dotenv::dotenv;
use rand::Rng as _;
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
use std::{collections::HashMap, sync::Arc};
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
        let author_id = msg.author.id.0;
        if let Some((guild_id, data_lock)) = msg.guild_id.zip(ctx.data.read().await.get::<Data>()) {
            let mut data = data_lock.write().await;
            let mut rng = rand::thread_rng();

            if cookies {
                data.cookies.give(author_id, rng.gen_range(0..2));
            }
            if levels {
                let xp = rng.gen_range(0..5);
                data.level.add_global_xp(author_id, xp);
                data.level.add_guild_xp(author_id, guild_id.0, xp);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let command = match interaction {
            Interaction::ApplicationCommand(command) => command,
            _ => return,
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
        let result = if let Some(guild) = ctx.data.read().await.get::<Config>().unwrap().home_server
        {
            id::GuildId(guild.into()).set_application_commands(&ctx.http, commands::commands).await
        } else {
            Command::set_global_application_commands(&ctx.http, commands::commands).await
        };
        if let Err(err) = result {
            eprintln!("Failed to register commands. More info:\n {err:#?}");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok(); // place variables from .env into this enviroment

    let token: String = env::var("DISCORD_TOKEN")
        .ok()
        .context("Put DISCORD_TOKEN=YourTokenHere into the file '.env' or the enviroment")?;

    let bot_id = if let Some((id, _)) = serenity::utils::parse_token(&token) {
        id
    } else {
        anyhow::bail!("DISCORD_TOKEN is invalid");
    };

    let mut config = Config::from_file("config.toml").unwrap_or_default();

    // If the App ID is supplied in the config file, use it.
    // Otherwise, use the Bot's user ID, which should be the
    // same on recently created applications.
    let application_id = config.application_id.take().map_or(bot_id.0, Into::into);
    config.application_id = application_id.try_into().ok();

    // create client
    let mut client = Client::builder(&token, GatewayIntents::GUILD_MESSAGES)
        .application_id(application_id)
        .event_handler(Handler)
        .await?;

    if let Ok(info) = client.cache_and_http.http.get_current_application_info().await {
        if let Some(team) = info.team {
            config.owners.extend(team.members.iter().map(|x| x.user.id.0));
        } else {
            config.owners.push(info.owner.id.0);
        }
    }

    {
        let mut client_data = client.data.write().await;
        client_data.insert::<Data>(Arc::new(RwLock::new(Data::new())));
        client_data.insert::<Config>(config);

        client_data.insert::<commands::tic_tac_toe::Running>(RwLock::new(Vec::new()));
        client_data.insert::<commands::tic_tac_toe::Queue>(RwLock::new(HashMap::new()));
    }

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        println!("Shutting Down...");
        shard_manager.lock().await.shutdown_all().await;
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
