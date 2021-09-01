pub mod info;
pub mod admin;
pub mod fun;
pub mod level_cookies;
use serenity::builder::CreateApplicationCommands;
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
}
