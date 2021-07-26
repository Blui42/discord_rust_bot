use tokio::time::{sleep, Duration};
use crate::{set_prefix, stringify_error};
use serenity::{
    model::channel::Message,
    prelude::*,
};


pub async fn kick(ctx: Context, msg: Message) -> Result<(), String>{
    if msg.is_private(){return Ok(())}
    // if the member doesn't have the Admin permission
    if ! msg.member(&ctx.http).await.map_err(stringify_error)?.permissions(&ctx).await.map_err(stringify_error)?.kick_members(){
        msg.channel_id.say(&ctx.http, "You don't have permission to kick").await.map_err(stringify_error)?;
        return Ok(())
    }
    if msg.mentions.len() == 0 {
        msg.channel_id.say(&ctx.http, "Noone was mentioned, so noone was kicked").await.map_err(stringify_error)?;
        return Ok(())
    }
    if msg.mentions_me(&ctx).await.unwrap_or(true){
        msg.channel_id.say(&ctx.http, "I won't kick myself").await.map_err(stringify_error)?;
        return Ok(())
    }
    let a = msg.guild(&ctx.cache).await.ok_or("Unknown Cache Error")?;
    for i in msg.mentions.iter(){
        if let Ok(_) = a.kick_with_reason(&ctx.http, i.id, &format!("Kicked by `{}`", msg.author.tag())).await{
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Kicked `{}`", i.tag())).await{
                eprintln!("An Error occured: {}", why)
            }
        }else{
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Can't kick `{}`", i.tag())).await{
                eprintln!("An Error occured: {}", why)
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    return Ok(())
}
pub async fn unban(ctx: Context, msg: Message) -> Result<(), String>{
    if msg.is_private(){return Ok(())}
    // if the member doesn't have the Admin permission
    if ! msg.member(&ctx.http).await.map_err(stringify_error)?.permissions(&ctx).await.map_err(stringify_error)?.ban_members(){
        msg.channel_id.say(&ctx.http, "You don't have permission to unban").await.map_err(stringify_error)?;
        return Ok(())
    }
    if msg.mentions.len() == 0 {
        msg.channel_id.say(&ctx.http, "Noone was mentioned, so noone was unbanned").await.map_err(stringify_error)?;
        return Ok(())
    }
    let a = msg.guild(&ctx.cache).await.ok_or("Unknown Cache Error")?;
    for i in msg.mentions.iter(){
        if let Ok(_) = a.unban(&ctx.http, i.id).await{
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Unbanned `{}`", i.tag())).await{
                eprintln!("An Error occured: {}", why)
            }
        }else{
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Can't unban `{}`", i.tag())).await{
                eprintln!("An Error occured: {}", why)
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    return Ok(())
}
pub async fn ban(ctx: Context, msg: Message) -> Result<(), String>{
    if msg.is_private(){return Ok(())}
    // if the member doesn't have the Admin permission
    if ! msg.member(&ctx.http).await.map_err(stringify_error)?.permissions(&ctx).await.map_err(stringify_error)?.ban_members(){
        msg.channel_id.say(&ctx.http, "You don't have permission to ban").await.map_err(stringify_error)?;
        return Ok(())
    }
    if msg.mentions.len() == 0 {
        msg.channel_id.say(&ctx.http, "Noone was mentioned, so noone was banned").await.map_err(stringify_error)?;
        return Ok(())
    }
    if msg.mentions_me(&ctx).await.unwrap_or(true){
        msg.channel_id.say(&ctx.http, "I won't ban myself").await.map_err(stringify_error)?;
        return Ok(())
    }
    let a = msg.guild(&ctx.cache).await.ok_or("Unknown Cache Error")?;
    for i in msg.mentions.iter(){
        if let Ok(_) = a.ban_with_reason(&ctx.http, i.id, 0, &format!("Banned by `{}`", msg.author.tag())).await{
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Banned `{}`", i.tag())).await{
                eprintln!("An Error occured: {}", why)
            }
        }else{
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Can't ban `{}`", i.tag())).await{
                eprintln!("An Error occured: {}", why)
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}
pub async fn prefix(mut ctx: Context, msg: Message) -> Result<(), String>{
    if msg.is_private(){return Ok(())}
    // if the member doesn't have the Admin permission
    if ! msg.member(&ctx.http).await.map_err(stringify_error)?.permissions(&ctx).await.map_err(stringify_error)?.administrator(){
        msg.channel_id.say(&ctx.http, "Only administrators can change the server prefix").await.map_err(stringify_error)?;
        return  Ok(())
    }
    let new_prefix = msg.content.replace("`", "");
    if new_prefix.len() > 5 {
        msg.channel_id.say(&ctx.http, "Prefixes can't be longer than 5 symbols.").await.map_err(stringify_error)?;
        return Ok(())
    }
    set_prefix(&msg, &mut ctx, &new_prefix).await;
    msg.channel_id.say(&ctx.http, format!("Successfully set prefix to `{}`", new_prefix)).await.map_err(stringify_error)?;
    return Ok(())
}
pub async fn delete(ctx: Context, msg: Message) -> Result<(), String>{
    if msg.is_private(){return Ok(())}
    // if the member doesn't have the manage messages permission
    if ! msg.member(&ctx.http).await.map_err(stringify_error)?.permissions(&ctx).await.map_err(stringify_error)?.manage_messages(){
        msg.channel_id.say(&ctx.http, "You can't delete messages").await.map_err(stringify_error)?;
        return Ok(())
    }
    
    let amount: u64 = msg.content.parse().unwrap_or(0);
    if amount == 0{
        msg.channel_id.say(&ctx.http, "Please specify a valid amount of messages to delete (1-100)").await.map_err(stringify_error)?;
        return Ok(());
    }
    let guild_channel = msg.channel(&ctx.cache).await.ok_or("Unknown Cache Error")?.guild().ok_or("Unknown Cache Error")?;
    let messages = guild_channel.messages(&ctx.http,   |retriever|{retriever.limit(amount+1)}  ).await.map_err(stringify_error)?;
    if let Err(_) = guild_channel.delete_messages(&ctx.http, messages).await{
        msg.channel_id.say(&ctx.http, "Please specify a valid amount of messages to delete (1-100)").await.map_err(stringify_error)?;
    }
    return Ok(())
}