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

/// Roll the dice
#[poise::command(slash_command)]
pub async fn roll(ctx: crate::utils::Context<'_>, rolls: u8, sides: u8) -> anyhow::Result<()> {
    if rolls == 0 {
        ctx.reply("Rolled no dice. (What did you expect?)").await?;
    } else if sides == 0 {
        ctx.reply("0-sided dice are too dangerous to use.").await?;
    } else if sides == 1 {
        ctx.reply("*Throws a ball*").await?;
    } else {
        let (total, min, max, summary) = roll_dice(rolls, sides);
        ctx.reply(format!("**Rolled {rolls} {sides}-sided dice.** \n**Result: `{total}`**\n Rolled {min}x1 and {max}x{sides} \n\n Detailed: ```\n{summary:?}\n```")).await?;
    }
    Ok(())
}

/// Flip a coin
#[poise::command(slash_command)]
pub async fn coin(ctx: crate::utils::Context<'_>) -> anyhow::Result<()> {
    ctx.say(flip_coin()).await?;
    Ok(())
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
