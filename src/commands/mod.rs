#![allow(clippy::unused_async)]
pub mod admin;
pub mod fun;
pub mod info;
pub mod level_cookies;
pub mod rock_paper_scissors;
pub mod tic_tac_toe;

use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::{
    builder::CreateApplicationCommands,
    model::{application::command::CommandOptionType, Permissions},
    prelude::*,
};
use std::borrow::Cow;

pub async fn respond_to<'a>(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &'a [CommandDataOption],
) -> anyhow::Result<Cow<'a, str>> {
    match command.data.name.as_str() {
        "roll" => fun::roll(options).await,
        "coin" => fun::coin().await,
        "id" => info::id(options, command.guild_id.as_ref()).await,
        "ttt" => tic_tac_toe::command(options, ctx, &command.user).await,
        "picture" => info::picture(options).await,
        "delete" => admin::delete(options, command.channel_id, ctx).await,
        "rockpaperscissors" => rock_paper_scissors::command(options, ctx, &command.user).await,
        x => Err(anyhow::anyhow!("Unknown Command: {x}")),
    }
}
#[allow(clippy::too_many_lines)]
#[rustfmt::skip]
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
                                .required(true)
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
                                .required(true)
                        })
                })
        })
        .create_application_command(|command| {
            command
                .name("picture")
                .description("Get a user's profile picture")
                .create_option(|option| {
                    option
                        .name("target")
                        .description("The User to get the profile picture of")
                        .kind(CommandOptionType::User)
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
                        .kind(CommandOptionType::Integer)
                        .min_int_value(0)
                        .max_int_value(255)
                        .required(true)
                })
                .create_option(|option| {
                    option
                        .name("sides")
                        .description("the amount of sides the thrown dice have")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(0)
                        .max_int_value(255)
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
                                .description("The field is numbered horizontally")
                                .kind(CommandOptionType::Integer)
                                .min_int_value(1)
                                .max_int_value(9)
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
        .create_application_command(|command| {
            command
                .name("delete")
                .description("Delete some messages")
                .create_option(|option| {
                    option
                        .name("count")
                        .description("The amount of messages to delete")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(100)
                        .required(true)
                })
                .default_member_permissions(Permissions::MANAGE_MESSAGES)
        })
        .create_application_command(|command| {
            command
                .name("rockpaperscissors")
                .name_localized("de", "scheresteinpapier")
                .description("Rock-Paper-Scissors!")
                .description_localized("de", "Schere-Stein-Papier!")
                .create_option(|option| {
                    option
                        .name("opponent")
                        .name_localized("de", "gegner")
                        .description("Who to play against")
                        .kind(CommandOptionType::User)
                })
                .create_option(|option| {
                    option
                        .name("thing")
                        .description("The thing you want to play")
                        .kind(CommandOptionType::String)
                        .add_string_choice_localized("Rock", "rock", [("de", "Stein")])
                        .add_string_choice_localized("Paper", "paper", [("de", "Papier")])
                        .add_string_choice_localized("Scissors", "scissors", [("de", "Schere")])
                })
        })
}
