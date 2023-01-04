use std::borrow::Cow;

use anyhow::{Context as _, Result};
use serenity::model::application::interaction::application_command::CommandDataOption;

fn roll_dice(rolls: u8, sides: u8) -> (u16, u8, u8, Vec<u8>) {
    let mut total: u16 = 0;
    let mut summary = Vec::with_capacity(sides.into());
    let mut min: u8 = 0;
    let mut max: u8 = 0;
    for _ in 0..rolls {
        let number = fastrand::u8(1..=sides);
        total += u16::from(number);
        summary.push(number);
        if number == sides {
            max += 1;
        }
        if number == 1 {
            min += 1;
        }
    }
    (total, min, max, summary)
}
pub async fn roll(options: &[CommandDataOption]) -> Result<Cow<'_, str>> {
    let rolls: u8 = options
        .get(0)
        .and_then(|arg| arg.value.as_ref())
        .and_then(serde_json::Value::as_u64)
        .and_then(|x| TryFrom::try_from(x).ok())
        .context("Missing rolls argument")?;
    let sides: u8 = options
        .get(1)
        .and_then(|arg| arg.value.as_ref())
        .and_then(serde_json::Value::as_u64)
        .and_then(|x| TryFrom::try_from(x).ok())
        .context("Missing sides argument")?;
    if rolls == 0 {
        Ok("Rolled no dice. (What did you expect?)".into())
    } else if sides == 0 {
        Ok("0-sided dice are too dangerous to use.".into())
    } else if sides == 1 {
        Ok("*Throws a ball*".into())
    } else {
        let (total, min, max, summary) = roll_dice(rolls, sides);
        Ok(format!("**Rolled {rolls} {sides}-sided dice.** \n**Result: `{total}`**\n Rolled {min}x1 and {max}x{sides} \n\n Detailed: ```\n{summary:?}\n```").into())
    }
}
pub async fn coin() -> Result<Cow<'static, str>> {
    Ok(flip_coin().into())
}

#[inline]
pub fn flip_coin() -> &'static str {
    match fastrand::i8(..) {
        -128..=-2 => "It landed tails!",
        -1 => "It didn't tip over",
        0 => "It fell under the table",
        1.. => "It landed heads",
    }
}
