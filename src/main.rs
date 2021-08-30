mod commands;
mod data;
use rand::{Rng, SeedableRng};
use data as myData;
use rand::rngs::SmallRng;
use commands::*;
use tokio::time::{sleep, Duration};
use std::env;
use dotenv::dotenv;
use serenity::{
    model::{
        channel::Message,
        gateway::Ready,
        interactions::Interaction,
        prelude::*
    },
    client::bridge::gateway::GatewayIntents,
    prelude::*
};


struct MyData;
impl TypeMapKey for MyData{
    type Value = myData::Data;
}

struct Handler;


async fn get_prefix(msg: &Message, ctx: &Context) -> Option<String>{
    // return "!" for PMs
    if msg.is_private(){return Some("!".to_string())}
    // get immutable reference to prefix variable
    return ctx.data
        .read().await
        .get::<MyData>()?
        .prefix
        .get(msg.guild_id?.0);
}
pub async fn set_prefix(msg: &Message, ctx: &mut Context, new_prefix: &str){
    // get mutable prefix variable
    if let Some(data) = ctx.data.write().await.get_mut::<MyData>(){
        if let Some(a) = msg.guild_id {
            data.prefix.set(a.0, new_prefix);
            return;
        }
    }
}
// give the user cookies and xp
pub async fn reward_user(msg: &Message, ctx: &mut Context){
    let author_id = msg.author.id.0;
    if let Some(data) = ctx.data.write().await.get_mut::<MyData>(){
        let mut rng = SmallRng::from_entropy();
        data.cookies.give(&author_id, rng.gen_range(0..2)); // cookies
        // xp
        let xp = rng.gen_range(0..5);
        data.level.add_xp(&author_id, &0, xp); // global xp
        if let Some(a) = msg.guild_id {
            data.level.add_xp(&author_id, &a.0, xp); // guild-specific xp
            return;
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, mut ctx: Context, mut msg: Message) {
        // ignore messages from bots
        if msg.author.bot {return}

        // get the prefix for the current guild
        let prefix: String = match get_prefix(&msg, &ctx).await {
            Some(a) => a,
            None => {
                set_prefix(&msg, &mut ctx, "!").await;
                "!".to_string()
            }
        };

        // gives xp and cookies to user
        reward_user(&msg, &mut ctx).await;

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
    async fn ready(&self, _ctx: Context, ready: Ready){
        println!("{} is connected!", ready.user.tag())
    }
}

#[tokio::main]
async fn main(){
    let data = myData::Data::new();
    dotenv().ok(); // place variables from .env into this enviroment

    // Only recieve messages
    let mut intent = GatewayIntents::empty();
        intent.set(GatewayIntents::GUILD_MESSAGES, true);
        intent.set(GatewayIntents::DIRECT_MESSAGES, true);

    let token = &env::var("DISCORD_TOKEN") // load DISCORD_TOKEN from enviroment
            .expect("Expected a token in the enviroment");  // panic if not present

    let application_id = env::var("APPLICATION_ID") // load APPLICATION_ID from enviroment
            .expect("Expected application id in the enviroment")  // panic if not present
            .parse::<u64>().expect("application id not parsable as number");

    // create client
    let mut client: Client = Client::builder(token)
        .intents(intent)
        .application_id(application_id)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    
    client.data.write().await.insert::<MyData>(data);

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
        if let Some(readable_data) = client_data.read().await.get::<MyData>(){
            readable_data.save();
        }else{
            eprintln!("Data to save was not accessible");
        }
    });
    
    if let Err(why) = client.start_autosharded().await {
        println!("Client error: {:?}", why);
    };
}
