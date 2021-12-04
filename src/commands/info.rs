use crate::stringify_error;
use serenity::{
    model::{channel::Message, interactions},
    prelude::*,
};


#[cfg(feature="legacy_commands")]
pub async fn ping(ctx: Context, msg: Message) -> Result<(), String>{
    msg.channel_id.say(&ctx.http, "pong!").await.map_err(stringify_error)?;
    Ok(())
}

#[cfg(feature="legacy_commands")]
pub async fn pic(ctx: Context, msg: Message) -> Result<(), String>{
    if msg.mentions.is_empty() {
        msg.channel_id.say(&ctx.http, msg.author.face()).await.map_err(stringify_error)?;
        return Ok(())
    }
    for i in msg.mentions.iter(){
        msg.channel_id.say(&ctx.http, i.face()).await.map_err(stringify_error)?;
    }
    Ok(())
}
#[cfg(feature="legacy_commands")]
pub async fn id(ctx: Context, msg: Message) -> Result<(), String>{
    for target in msg.content.split_whitespace(){
        if !(target.starts_with('<') && target.ends_with('>')){
            msg.channel_id.say(&ctx.http, "That's not a valid target. Mention a user, role, channel, etc").await.map_err(stringify_error)?;
            continue;
        }
        let mut id = target.to_string(); 
        for i in &["<",">","!","#","@","&"]{
            id = id.replace(i, "");
        }
        msg.channel_id.say(&ctx.http, id).await.map_err(stringify_error)?;
    }
    Ok(())
}
#[cfg(feature="legacy_commands")]
pub async fn get_id_command(options: &Vec::<interactions::application_command::ApplicationCommandInteractionDataOption>) -> Option<String>{
    let target = options.get(0)?.value.as_ref()?.as_str()?;
    if !(target.starts_with('<') && target.ends_with('>')){
        return Some("That's not a valid target. Mention a user, role, channel, etc".to_string());
    }
    let mut id = target.to_string(); 
    for i in &["<",">","!","#","@","&"]{
        id = id.replace(i, "");
    }
    Some(id)
}
