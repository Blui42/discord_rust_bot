#![allow(clippy::unused_async)]
pub mod admin;
pub mod fun;
pub mod info;
pub mod level_cookies;
pub mod rock_paper_scissors;
pub mod tic_tac_toe;

use std::borrow::Cow;

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, Permissions,
};

pub async fn respond_to<'a>(
    ctx: &Context,
    command: &'a CommandInteraction,
) -> anyhow::Result<Cow<'a, str>> {
    let options = command.data.options.as_slice();
    let resolved_options_vec = &command.data.options();
    let resolved_options = resolved_options_vec.as_slice();
    match command.data.name.as_str() {
        "roll" => fun::roll(options).await,
        "coin" => fun::coin().await,
        "id" => info::id(options, command.guild_id.as_ref()).await,
        "ttt" => tic_tac_toe::command(resolved_options, ctx, &command.user).await,
        "picture" => info::picture(resolved_options).await,
        "delete" => admin::delete(options, command.channel_id, ctx).await,
        "rockpaperscissors" => {
            rock_paper_scissors::command(resolved_options, ctx, &command.user).await
        }
        x => Err(anyhow::anyhow!("Unknown Command: {x}")),
    }
}
pub fn commands() -> Vec<CreateCommand> {
    #[allow(clippy::enum_glob_use)]
    use CommandOptionType::*;
    let option = CreateCommandOption::new;
    let required_option = |kind, name, description| option(kind, name, description).required(true);
    vec![
        CreateCommand::new("id")
            .description("Get the ID of the mentioned user/role/channel")
            .set_options(vec![
                option(SubCommand, "server", "Get ID of the server you're on"),
                option(SubCommand, "user", "Get ID of a user or role").add_sub_option(
                    required_option(Mentionable, "target", "The user or role to get the ID of"),
                ),
                option(SubCommand, "channel", "Get the ID of a Channel").add_sub_option(
                    required_option(Channel, "target", "The Channel to get the ID of"),
                ),
            ]),
        CreateCommand::new("picture")
            .description("Get a user's profile picture")
            .add_option(required_option(User, "target", "The User to get the profile picture of")),
        CreateCommand::new("roll").description("Roll the dice").set_options(vec![
            required_option(Integer, "rolls", "The amount of dice to roll")
                .min_int_value(0)
                .max_int_value(255),
            required_option(Integer, "sides", "the number of sides the dice have")
                .min_int_value(0)
                .max_int_value(255),
        ]),
        CreateCommand::new("coin").description("Toss a coin"),
        CreateCommand::new("ttt").description("Tic Tac Toe").set_options(vec![
            option(SubCommand, "start", "Start a new game of Tic Tac Toe"),
            option(SubCommand, "set", "Set your marker on the playing field").add_sub_option(
                required_option(Integer, "field", "The field is numbered horizontally")
                    .min_int_value(1)
                    .max_int_value(9),
            ),
            option(SubCommand, "cancel", "Cancel an upcoming or ongoing game").add_sub_option(
                option(User, "opponent", "The opponent of the game you want to cancel"),
            ),
        ]),
        CreateCommand::new("delete")
            .description("Delete some Messages")
            .default_member_permissions(Permissions::MANAGE_MESSAGES)
            .add_option(
                required_option(Integer, "count", "The amount of messages to delete")
                    .min_int_value(1)
                    .max_int_value(100),
            ),
        CreateCommand::new("rockpaperscissors")
            .name_localized("de", "scheresteinpapier")
            .description("Rock-Paper-Scissors!")
            .description_localized("de", "Schere-Stein-Papier!")
            .set_options(vec![
                required_option(User, "opponent", "Who to play against")
                    .name_localized("de", "gegner"),
                required_option(String, "thing", "The thing you want to play")
                    .add_string_choice_localized("Rock", "rock", [("de", "Stein")])
                    .add_string_choice_localized("Paper", "paper", [("de", "Papier")])
                    .add_string_choice_localized("Scissors", "scissors", [("de", "Schere")]),
            ]),
    ]
}
