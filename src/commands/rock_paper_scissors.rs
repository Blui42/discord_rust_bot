use std::str::FromStr;

use poise::serenity_prelude::User;
use tokio::time::Instant;

/// Play Rock-Paper-Scissors against another user
#[poise::command(slash_command, rename = "rockpaperscissors")]
pub async fn rock_paper_scissors(
    ctx: crate::utils::Context<'_>,
    opponent: User,
    thing: Rps,
) -> anyhow::Result<()> {
    let author = ctx.author();
    if opponent == *author {
        ctx.reply("You can't play againt yourself").await?;
        return Ok(());
    }
    let mut map = ctx.data().rps.lock().await;
    if let Some((a, timestamp)) = map.remove(&(author.id, opponent.id)) {
        let duration_ms = Instant::now().duration_since(timestamp).as_millis();
        if duration_ms < 5000 {
            let result = thing.match_against(a);
            // 202f: thin non-breaking whitespace
            ctx.reply(format!("{result}\nIt took you {duration_ms}\u{202f}ms to respond.")).await?;
            return Ok(());
        }
    }
    map.insert((opponent.id, author.id), (thing, Instant::now()));
    drop(map);
    ctx.reply("Waiting for your opponent...").await?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, poise::ChoiceParameter)]
pub enum Rps {
    Rock,
    Paper,
    Scissors,
}

impl Rps {
    pub fn match_against(self, other: Self) -> &'static str {
        if self == other {
            "It's a tie!"
        } else if self == other >> () {
            "You won!"
        } else {
            "You lost!"
        }
    }
}

impl std::ops::Shr<()> for Rps {
    type Output = Self;

    fn shr(self, (): ()) -> Self::Output {
        match self {
            Self::Rock => Self::Paper,
            Self::Paper => Self::Scissors,
            Self::Scissors => Self::Rock,
        }
    }
}

impl std::ops::Shl<()> for Rps {
    type Output = Self;

    fn shl(self, (): ()) -> Self::Output {
        match self {
            Self::Rock => Self::Scissors,
            Self::Scissors => Self::Paper,
            Self::Paper => Self::Rock,
        }
    }
}

impl FromStr for Rps {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rock" => Ok(Self::Rock),
            "paper" => Ok(Self::Paper),
            "scissors" => Ok(Self::Scissors),
            _ => Err(()),
        }
    }
}
