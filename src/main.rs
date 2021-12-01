mod commands;
mod data;
use commands::*;
use tokio::time::{sleep, Duration};
use std::env;
use dotenv::dotenv;
use serenity::{
    model::{
        channel::Message,
        gateway::Ready,
        interactions::{
            Interaction,
            application_command::ApplicationCommand,
        },
        prelude::*
    },
    client::bridge::gateway::GatewayIntents,
    prelude::*
};
use data::{
    Data,
    prefix::Prefix,
    config::Config
};


struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, mut ctx: Context, mut msg: Message) {
        // ignore messages from bots
        if msg.author.bot {return}

        // get the prefix for the current guild
        let prefix: String = match data::prefix::get_prefix(&msg, &ctx).await {
            Some(a) => a,
            None => {
                data::prefix::set_prefix(&msg, &mut ctx, "!").await;
                "!".to_string()
            }
        };

        // gives xp and cookies to user
        data::reward_user(&msg, &mut ctx).await;

        if ! msg.content.starts_with(&prefix) {
            // print out the prefix if the bot is mentioned
            if let Ok(true) = msg.mentions_me(&ctx.http).await {
                if let Err(why) = msg.channel_id.say(&ctx.http, format!("Hi, {}! The current prefix is {}", msg.author, prefix)).await{
                    eprintln!("An Error occured: {}", why)
                }
            }
            return
        }
        // split command off of the message, make command lowercase for case insensitivity
        msg.content = msg.content.replacen(&prefix, "", 1); 
        let command = msg.content.split_whitespace().next().unwrap_or(" ").to_lowercase();
        msg.content = msg.content.replacen(&command, "", 1).replacen(" ", "", 1);
        // matches for correct command, hands over ctx, msg
        let command_result = match command.as_str() {
            // "cmd"    => category::cmd(ctx, msg).await,       // Comment on what this does
            "ping"      => info::ping(ctx, msg).await,          // send pong!
            "prefix"    => admin::prefix(ctx, msg).await,       // change prefix
            "kick"      => admin::kick(ctx, msg).await,         // kick all users mentioned in the message
            "ban"       => admin::ban(ctx, msg).await,          // ban all users mentioned in the message 
            "unban"     => admin::unban(ctx, msg).await,        // unban all users mentioned in the message 
            "pic"       => info::pic(ctx, msg).await,           // send profile picture of message.author or all users mentioned
            "delete"    => admin::delete(ctx, msg).await,       // delete specified amount of messages
            "id"        => info::id(ctx, msg).await,            // get id of all mentioned users/roles/channels, etc.
            "roll"      => fun::roll(ctx, msg).await,           // roll dice according to arg
            "coin"      => fun::coin(ctx, msg).await,           // flip a coin
            _ => Ok(()),
        };
        if let Err(why) = command_result {
            eprintln!("An Error occured: {}", why)
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let response = match command.data.name.as_str() {
                "roll" => fun::roll_command(&command.data.options).await,
                "coin" => fun::coin_command().await,
                "id" => info::get_id_command(&command.data.options).await,
                _ => None
            };
            if let Some(a) = response {
                if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(interactions::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(a))
                }).await {
                    eprintln!("An Error occured: {}", why)
                }
            }else{
                if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(interactions::InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("An Error occured."))
                }).await {
                    eprintln!("An Error occured: {}", why)
                }
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready){
        println!("{} is connected!", ready.user.tag());
        let _ = ApplicationCommand::set_global_application_commands(&ctx.http, commands).await;
        // let _ = id::GuildId(792489181774479400).set_application_commands(&ctx.http, commands).await;
    }
}

#[tokio::main]
async fn main(){
    dotenv().ok(); // place variables from .env into this enviroment
    
    // Only recieve messages
    let mut intent = GatewayIntents::empty();
    intent.set(GatewayIntents::GUILD_MESSAGES, true);
    intent.set(GatewayIntents::DIRECT_MESSAGES, true);
    
    let token = env::var("DISCORD_TOKEN") // load DISCORD_TOKEN from enviroment
    .expect("Expected a token in the enviroment");  // panic if not present
    
    let config = data::config::Config::from_file("config.toml").unwrap_or_default();

    // Try to read Application ID from config.toml.
    // On failure, try to derive Application ID from bot token. 
    let application_id = match config.application_id {
        Some(a) => a,
        None => {
            serenity::client::parse_token(&token)
            .expect("Application ID was not given and could not be derived from token.")
            .bot_user_id.0
        }
    };
    // create client
    let mut client: Client = Client::builder(&token)
    .intents(intent)
    .application_id(application_id)
    .event_handler(Handler)
    .await
    .expect("Err creating client");
    
    {
        let mut client_data = client.data.write().await;
        let data = Data::new();
        client_data.insert::<Data>(data);
        let prefix = Prefix::new("prefix.json".to_string());
        client_data.insert::<Prefix>(prefix);
        client_data.insert::<Config>(config);
    }

    let shard_manager = client.shard_manager.clone();
    let client_data = client.data.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        println!("Shutting Down...");
        shard_manager.lock().await.shutdown_all().await;
        println!("Successfully Shut Down");
    });

    // automatically save every 10 minutes
    tokio::spawn(async move {
        sleep(Duration::from_secs(600)).await;
        println!("Preparing to save...");
        if let Some(readable_data) = client_data.read().await.get::<Data>(){
            readable_data.save();
        }else{
            eprintln!("Data to save was not accessible");
        }
        if let Some(prefix) = client_data.read().await.get::<Prefix>(){
            prefix.save();
        }else{
            eprintln!("Data to save was not accessible");
        }
    });
    
    if let Err(why) = client.start_autosharded().await {
        println!("Client error: {:?}", why);
    };
}
