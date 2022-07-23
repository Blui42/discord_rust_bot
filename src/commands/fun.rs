use std::borrow::Cow;

use anyhow::{Context as _, Result};
use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
};
use serenity::{
    model::{application::interaction::application_command::CommandDataOption, channel::Message},
    prelude::*,
};

#[cfg(feature = "legacy_commands")]
pub async fn roll(ctx: Context, msg: Message) -> Result<()> {
    let (rolls, sides): (u8, u8) = msg.content.split_once('d').map_or((1, 6), |(x, y)| {
        (x.parse().unwrap_or(1), y.parse().unwrap_or(6))
    });
    if (sides < 2) || (rolls == 0) {
        msg.channel_id
            .say(&ctx.http, "Isn't that a bit pointless?")
            .await?;
        return Ok(());
    }
    let (total, min, max, summary) = roll_dice(rolls, sides);
    let response: String = format!(
        "**Result: `{total}`**\n Rolled {min}x1 and {max}x{sides} \n\n Detailed: ```{summary}```"
    );
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Rolled {rolls}d{sides}."))
                    .description(response)
                    .colour(0xff)
            })
        })
        .await?;
    Ok(())
}
fn roll_dice(rolls: u8, sides: u8) -> (u16, u8, u8, String) {
    let between = Uniform::new_inclusive(1, sides);
    let mut rng = thread_rng();
    let mut total: u16 = 0;
    let mut summary: String = String::new();
    let mut min: u8 = 0;
    let mut max: u8 = 0;
    for roll in 1..=rolls {
        let number: u8 = between.sample(&mut rng);
        total += u16::from(number);
        summary += &number.to_string();
        if roll != rolls {
            summary += ", ";
        }
        if number == sides {
            max += 1;
        }
        if number == 1 {
            min += 1;
        }
    }
    (total, min, max, summary)
}
pub async fn roll_command(options: &[CommandDataOption]) -> Result<Cow<'_, str>> {
    let rolls: i64 = options
        .get(0)
        .context("missing rolls field")?
        .value
        .as_ref()
        .context("missing rolls field")?
        .as_i64()
        .context("rolls was not a number")?;
    let sides: i64 = options
        .get(1)
        .context("missing sides field")?
        .value
        .as_ref()
        .context("missing sides field")?
        .as_i64()
        .context("sides was not a number")?;
    if rolls < 0 || sides < 0 {
        Ok("<= !em pleH <=".into())
    } else if rolls == 0 {
        Ok("Rolled no dice. (What did you expect?)".into())
    } else if sides == 0 {
        Ok("0-sided dice are too dangerous to use.".into())
    } else if sides == 1 {
        Ok("*Throws a ball*".into())
    } else if rolls > 255 || sides > 255 {
        Ok("A number that I'm too lazy to calculate (Try numbers 255 and below)".into())
    } else {
        let (total, min, max, summary) = roll_dice(rolls.to_le_bytes()[0], sides.to_le_bytes()[0]);
        Ok(format!("**Rolled {rolls} {sides}-sided dice.** \n**Result: `{total}`**\n Rolled {min}x1 and {max}x{sides} \n\n Detailed: ```{summary}```").into())
    }
}
pub async fn coin_command() -> Result<Cow<'static, str>> {
    Ok(flip_coin().into())
}

#[inline]
pub fn flip_coin() -> &'static str {
    let number: u8 = random();
    if number > 128 {
        "It landed tails!"
    } else if number < 127 {
        "It landed heads!"
    } else if number == 127 {
        "It didn't tip over!"
    } else {
        "It fell under the table!"
    }
}
#[cfg(feature = "legacy_commands")]
pub async fn coin(ctx: Context, msg: Message) -> Result<()> {
    msg.channel_id.say(&ctx.http, &flip_coin()).await?;
    Ok(())
}
