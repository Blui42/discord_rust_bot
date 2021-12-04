pub mod info;
pub mod admin;
pub mod fun;
pub mod level_cookies;
use serenity::builder::CreateApplicationCommands;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

#[allow(dead_code)]
pub fn stringify_error<X: std::fmt::Debug>(error: X) -> String{
    return format!("An Error occured: {:?}", error)
}
pub fn commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    commands
    .create_application_command(|command| {
        command
        .name("id")
        .description("Get the ID of the mentioned user/role/channel")
        .create_option(|option| {
            option
            .name("target")
            .description("user/role/channel to get the ID from")
            .kind(ApplicationCommandOptionType::Mentionable)
            .required(true)
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
            .kind(ApplicationCommandOptionType::Integer)
            .required(true)
        })
        .create_option(|option| {
            option
            .name("sides")
            .description("the amount of sides the thrown dice have")
            .kind(ApplicationCommandOptionType::Integer)
            .required(true)
        })
    })
    .create_application_command(|command| {
        command
        .name("coin")
        .description("Toss a coin")
    })
}
#[cfg(feature="legacy_commands")]
pub async fn parse_command(prefix: &str, mut msg: Message, ctx: Context){
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
        // "cmd" => category::cmd(ctx, msg).await, // Comment on what this does
        "ping"   => info::ping(ctx, msg).await,    // send pong!
        "prefix" => admin::prefix(ctx, msg).await, // change prefix
        "kick"   => admin::kick(ctx, msg).await,   // kick all users mentioned in the message
        "ban"    => admin::ban(ctx, msg).await,    // ban all users mentioned in the message 
        "unban"  => admin::unban(ctx, msg).await,  // unban all users mentioned in the message 
        "pic"    => info::pic(ctx, msg).await,     // send profile picture of msg.author or all users mentioned
        "delete" => admin::delete(ctx, msg).await, // delete specified amount of messages
        "id"     => info::id(ctx, msg).await,      // get id of all mentioned users/roles/channels, etc.
        "roll"   => fun::roll(ctx, msg).await,     // roll dice according to arg
        "coin"   => fun::coin(ctx, msg).await,     // flip a coin
        _ => Ok(()),
    };
    if let Err(why) = command_result {
        eprintln!("An Error occured: {}", why)
    }
}