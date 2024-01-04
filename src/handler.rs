use crate::commands;
use crate::data::{config::Config, Data};
use serenity::{
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    client::{Context, EventHandler},
    model::{
        application::{Command, Interaction},
        channel::Message,
        gateway::Ready,
        id::GuildId,
    },
};

pub struct Handler;

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
        let author_id = msg.author.id.get();
        if let Some((guild_id, data_lock)) = msg.guild_id.zip(ctx.data.read().await.get::<Data>()) {
            let mut data = data_lock.write().await;

            if cookies {
                data.cookies.give(author_id, fastrand::u64(0..2));
            }
            if levels {
                let xp = fastrand::u64(0..5);
                data.level.add_global_xp(author_id, xp);
                data.level.add_guild_xp(guild_id.get(), author_id, xp);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());
        let result = if let Some(guild) = ctx.data.read().await.get::<Config>().unwrap().home_server
        {
            Command::set_global_commands(&ctx.http, Vec::new()).await.ok();
            GuildId::from(guild).set_commands(&ctx.http, commands::commands()).await
        } else {
            Command::set_global_commands(&ctx.http, commands::commands()).await
        };
        if let Err(err) = result {
            eprintln!("Failed to register commands. More info:\n {err:#?}");
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else { return };
        let response = match commands::respond_to(&ctx, &command).await {
            Ok(msg) => CreateInteractionResponseMessage::new().content(msg),
            Err(e) => {
                eprintln!("------------\n{e:?}\n------------\n{command:?}\n------------");
                CreateInteractionResponseMessage::new().content(
                format!("An Error occurred: {e}\n\
                If you find a consistent way to cause this error, please report it to my support discord.")).ephemeral(true)
            }
        };
        command
            .create_response(&ctx.http, CreateInteractionResponse::Message(response))
            .await
            .unwrap_or_else(|why| eprintln!("An Error occurred: {why}"));
    }
}

impl Handler {}
