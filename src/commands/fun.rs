use std::borrow::Cow;

use anyhow::{Context as _, Result};
use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
};
use serenity::model::application::interaction::application_command::CommandDataOption;

fn roll_dice(rolls: u8, sides: u8) -> (u16, u8, u8, Vec<u8>) {
    let between = Uniform::new_inclusive(1, sides);
    let mut rng = thread_rng();
    let mut total: u16 = 0;
    let mut summary = Vec::with_capacity(sides.into());
    let mut min: u8 = 0;
    let mut max: u8 = 0;
    for _ in 0..rolls {
        let number: u8 = between.sample(&mut rng);
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
        Ok(format!("**Rolled {rolls} {sides}-sided dice.** \n**Result: `{total}`**\n Rolled {min}x1 and {max}x{sides} \n\n Detailed: ```{summary:?}```").into())
    }
}
pub async fn coin() -> Result<Cow<'static, str>> {
    Ok(flip_coin().into())
}

#[inline]
pub fn flip_coin() -> &'static str {
    match thread_rng().gen::<i8>() {
        -128..=-2 => "It landed tails!",
        -1 => "It didn't tip over",
        0 => "It fell under the table",
        1.. => "It landed heads",
    }
}
