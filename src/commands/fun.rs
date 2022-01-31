use crate::stringify_error;
use serenity::{
    model::{channel::Message, interactions::application_command::ApplicationCommandInteractionDataOption},
    prelude::*,
};
use rand::{distributions::{Distribution, Uniform}, prelude::*};

#[cfg(feature="legacy_commands")]
pub async fn roll(ctx: Context, msg: Message) -> Result<(), String>{
    let value = msg.content.replace(" ", "");
    let mut an_iterator = value.split('d');
    let rolls: u8 = an_iterator.next().unwrap_or("1").parse().unwrap_or(1);
    let sides: u8 = an_iterator.next().unwrap_or("6").parse().unwrap_or(6);
    drop(an_iterator);
    drop(value);
    if (sides < 2) || (rolls == 0){
        msg.channel_id.say(&ctx.http, "Isn't that a bit pointless?").await.map_err(stringify_error)?;
        return Ok(())
    }
    let (total, min, max, summary) = roll_dice(rolls, sides);
    let response: String = format!("**Result: `{}`**\n Rolled {}x1 and {}x{} \n\n Detailed: ```{}```", total, min, max, sides, summary);
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title(format!("Rolled {}d{}.", rolls, sides)).description(response).colour(0x0000ff)
        })
    }).await.map_err(stringify_error)?;
    Ok(())
}
fn roll_dice(rolls: u8, sides: u8) -> (u16, u8, u8, String) {
    let between = Uniform::new_inclusive(1, sides);
    let mut rng = thread_rng();
    let mut total: u16 = 0;
    let mut summary: String = "".to_string();
    let mut min: u8 = 0;
    let mut max: u8 = 0;
    for i in 1..=rolls{
        let number: u8 = between.sample(&mut rng);
        total += number as u16;
        summary += &number.to_string();
        if i != rolls{
            summary += ", "
        }
        if number == sides{
            max += 1
        }
        if number == 1{
            min += 1
        }
    }
    (total, min, max, summary)
}
pub async fn roll_command(options: &[ApplicationCommandInteractionDataOption]) -> Option<String> {
    let rolls: i64 = options.get(0)?.value.as_ref()?.as_i64()?;
    let sides: i64 = options.get(1)?.value.as_ref()?.as_i64()?;
    if rolls < 0 || sides < 0 {
        return Some("<= !em pleH <=".to_string())
    }
    if rolls == 0 {
        return Some("Rolled no dice. (What did you expect?)".to_string())
    }
    if sides == 0 {
        return Some("0-sided dice are too dangerous to use.".to_string())
    }
    if sides == 1 {
        return Some("*Throws a ball*".to_string())
    }
    if rolls > 255 || sides > 255 {
        return Some("A number that I'm too lazy to calculate (Try numbers 255 and below)".to_string())
    }
    let (total, min, max, summary) = roll_dice(rolls as u8, sides as u8);
    Some(format!("**Rolled {} {}-sided dice.**\n**Result: `{}`**\n Rolled {}x1 and {}x{} \n\n Detailed: ```{}```", rolls, sides, total, min, max, sides, summary))
}
pub async fn coin_command() -> Option<String> {
    Some(flip_coin())
}
pub fn flip_coin() -> String {
    let number: u8 = random();
    if number >  128 {return "It landed tails!".to_string()}
    if number <  127 {return "It landed heads!".to_string()}
    if number == 127 {return "It didn't tip over!".to_string()}
  /*if number == 128 return*/"It fell under the table!".to_string()
}
#[cfg(feature="legacy_commands")]
pub async fn coin(ctx: Context, msg: Message) -> Result<(), String>{
    msg.channel_id.say(&ctx.http, &flip_coin()).await.map_err(stringify_error)?;
    Ok(())
}
