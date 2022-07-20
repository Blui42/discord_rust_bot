#![allow(clippy::unused_async)]
pub mod admin;
pub mod fun;
pub mod info;
pub mod level_cookies;
#[cfg(feature = "tic_tac_toe")]
pub mod tic_tac_toe;
use serenity::builder::CreateApplicationCommands;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::channel::Message;

pub fn commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    commands
        .create_application_command(|command| {
            command
                .name("id")
                .description("Get the ID of the mentioned user/role/channel")
                .create_option(|option| {
                    option
                        .name("server")
                        .description("Get ID of the server you're on")
                        .kind(CommandOptionType::SubCommand)
                })
                .create_option(|option| {
                    option
                        .name("user")
                        .description("Get ID of a user or role")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|sub_option| {
                            sub_option
                                .name("target")
                                .description("The user or role to get the ID of")
                                .kind(CommandOptionType::Mentionable)
                        })
                })
                .create_option(|option| {
                    option
                        .name("channel")
                        .description("Get the user of a Channel")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|sub_option| {
                            sub_option
                                .name("target")
                                .description("The Channel to get the ID of")
                                .kind(CommandOptionType::Channel)
                        })
                })
        })
        .create_application_command(|command| {
            command
                .name("roll")
                .description("Rolls dice")
                .create_option(|option| {
                    option
                        .name("rolls")
                        .description("The amount of dice to roll")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
                .create_option(|option| {
                    option
                        .name("sides")
                        .description("the amount of sides the thrown dice have")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
        })
        .create_application_command(|command| command.name("coin").description("Toss a coin"))
        .create_application_command(|command| {
            command
                .name("ttt")
                .description("Tic Tac Toe")
                .create_option(|option| {
                    option
                        .name("start")
                        .description("Start a new game")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|sub_option| {
                            sub_option
                                .name("opponent")
                                .description("Your Opponent")
                                .kind(CommandOptionType::User)
                        })
                })
                .create_option(|option| {
                    option
                        .name("set")
                        .description("Set your marker on the playing field")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|sub_option| {
                            sub_option
                                .name("field")
                                .description("Should be from 1-9")
                                .kind(CommandOptionType::Integer)
                                .required(true)
                        })
                })
                .create_option(|option| {
                    option
                        .name("cancel")
                        .description("Cancel a upcoming or ongoing game")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|sub_option| {
                            sub_option
                                .name("opponent")
                                .description("The opponent of the game you want to cancel")
                                .kind(CommandOptionType::User)
                        })
                })
        })
}
#[cfg(feature = "legacy_commands")]
pub async fn parse(prefix: &str, mut msg: Message, ctx: Context) {
    if !msg.content.starts_with(&prefix) {
        // print out the prefix if the bot is mentioned
        if let Ok(true) = msg.mentions_me(&ctx.http).await {
            if let Err(why) = msg
                .channel_id
                .say(
                    &ctx.http,
                    format!("Hi, {}! The current prefix is {}", msg.author, prefix),
                )
                .await
            {
                eprintln!("An Error occured: {}", why);
            }
        }
        return;
    }
    // split command off of the message, make command lowercase for case insensitivity
    msg.content = msg.content.replacen(&prefix, "", 1);
    let command = msg
        .content
        .split_whitespace()
        .next()
        .unwrap_or(" ")
        .to_lowercase();
    msg.content = msg.content.replacen(&command, "", 1).replacen(" ", "", 1);
    // matches for correct command, hands over ctx, msg
    let command_result = match command.as_str() {
        // "cmd" => category::cmd(ctx, msg).await, // Comment on what this does
        "ping" => info::ping(ctx, msg).await,      // send pong!
        "prefix" => admin::prefix(ctx, msg).await, // change prefix
        "kick" => admin::kick(ctx, msg).await,     // kick all users mentioned in the message
        "ban" => admin::ban(ctx, msg).await,       // ban all users mentioned in the message
        "unban" => admin::unban(ctx, msg).await,   // unban all users mentioned in the message
        "pic" => info::pic(ctx, msg).await, // send profile picture of msg.author or all users mentioned
        "delete" => admin::delete(ctx, msg).await, // delete specified amount of messages
        "id" => info::id(ctx, msg).await,   // get id of all mentioned users/roles/channels, etc.
        "roll" => fun::roll(ctx, msg).await, // roll dice according to arg
        "coin" => fun::coin(ctx, msg).await, // flip a coin
        _ => Ok(()),
    };
    if let Err(why) = command_result {
        eprintln!("An Error occured: {:?}", why);
    }
}
