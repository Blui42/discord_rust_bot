use std::{borrow::Cow, collections::HashMap, str::FromStr, sync::Arc};

use anyhow::{bail, Result};
use serenity::all::{Context, ResolvedOption, ResolvedValue, User, UserId};
use serenity::prelude::TypeMapKey;
use tokio::{sync::Mutex, time::Instant};

pub async fn command<'a>(
    options: &'a [ResolvedOption<'_>],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'static, str>> {
    let Some(ResolvedValue::User(opponent, _)) = options.get(0).map(|x| &x.value) else {
        bail!("No user arg.");
    };
    if *opponent == user {
        return Ok("You can't play against yourself".into());
    }

    let Some(ResolvedValue::String(play)) = options.get(1).map(|x| &x.value) else {
        bail!("No option arg.");
    };
    let Ok(play) = play.parse::<Rps>() else {
        bail!("Invalid input string");
    };
    let mutex = ctx.data.read().await.get::<Queue>().unwrap().clone();
    let mut queue = mutex.lock().await;
    if let Some((a, timestamp)) = queue.remove(&(user.id, opponent.id)) {
        let duration_ms = Instant::now().duration_since(timestamp).as_millis();
        if duration_ms < 5000 {
            let result = play.match_against(a);
            // 202f: thin non-breaking whitespace
            return Ok(format!("{result}\nIt took you {duration_ms}\u{202f}ms to respond.").into());
        }
    }
    queue.insert((opponent.id, user.id), (play, Instant::now()));
    Ok("Waiting for your opponent...".into())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
pub struct Queue;

impl TypeMapKey for Queue {
    type Value = Arc<Mutex<HashMap<(UserId, UserId), (Rps, Instant)>>>;
}
