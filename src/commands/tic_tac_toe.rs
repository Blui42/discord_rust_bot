#![cfg(feature = "tic_tac_toe")]
use anyhow::{bail, Context as _, Result};
use serenity::{
    model::{
        application::interaction::application_command::{
            CommandDataOption, CommandDataOptionValue,
        },
        id::UserId,
        prelude::*,
    },
    prelude::*,
};
use std::{borrow::Cow, fmt};
use tokio::sync::RwLock;

pub async fn command<'a>(
    options: &'a [CommandDataOption],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'a, str>> {
    let subcommand = options.get(0).context("get subcommand")?;
    match subcommand.name.as_str() {
        "start" => start_game(subcommand.options.as_slice(), ctx, user).await,
        "set" => mark_field(subcommand.options.as_slice(), ctx, user).await,
        "cancel" => cancel_game(subcommand.options.as_slice(), ctx, user).await,
        _ => Ok("Unknown Subcommand".into()),
    }
}

pub async fn make_request(opponent: User, ctx: &Context, user: &User) -> Result<Cow<'static, str>> {
    if opponent.id == user.id {
        return Ok("That would be kind of sad".into());
    }
    let data = ctx.data.read().await;
    let current_games = data
        .get::<Running>()
        .context("get running games")?
        .read()
        .await;
    for game in current_games.iter() {
        if let Some(oppnent) = game.opponent(user.id) {
            return Ok(format!(
                "You are already in a game against {}!",
                oppnent.to_user(&ctx.http).await?.tag()
            )
            .into());
        }
        if game.has_player(opponent.id) {
            return Ok(format!("{} is already in a game!", opponent.tag()).into());
        }
    }
    drop(current_games);
    let game_queue = data.get::<Queue>().context("get game queue")?.read().await;
    for game in game_queue.iter() {
        if game.player_1 == user.id {
            return Ok(format!(
                "You have already requested a game against <@{}>!",
                game.player_2
            )
            .into());
        }
    }
    drop(game_queue);
    let mut game_queue = data.get::<Queue>().context("Get game queue")?.write().await;
    game_queue.push(TicTacToe::new(user.id, opponent.id));
    Ok(format!("You challenged {}!", opponent.mention()).into())
}

pub async fn cancel_game<'a>(
    options: &'a [CommandDataOption],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'a, str>> {
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        match a.resolved.as_ref().context("get field `opponent`")? {
            CommandDataOptionValue::User(opponent, _) => Some(&opponent.id),
            x => bail!(
                "In {}/{}: opponent was of incorrect type: Expected User, got {x:?}```",
                file!(),
                line!(),
            ),
        }
    } else {
        None
    };

    let data = ctx.data.read().await;
    let game_queue = data.get::<Queue>().context("get game queue")?.read().await;
    if let Some(index) = find_game_with_either(game_queue.iter(), user, opponent) {
        drop(game_queue);
        data.get::<Queue>()
            .context("get game queue")?
            .write()
            .await
            .swap_remove(index);
        return Ok("Cancelled game.".into());
    }

    let running_games = data
        .get::<Running>()
        .context("get running games")?
        .read()
        .await;
    if let Some(index) = find_game_with_either(running_games.iter(), user, opponent) {
        drop(running_games);
        data.get::<Running>()
            .context("get running games")?
            .write()
            .await
            .swap_remove(index);
        return Ok("You gave up.".into());
    }
    Ok("There was no game to cancel".into())
}

fn find_game_with_either<'a>(
    game_queue: impl Iterator<Item = &'a TicTacToe>,
    user: &User,
    opponent: Option<&UserId>,
) -> Option<usize> {
    game_queue
        .enumerate()
        .find(|(_, game)| {
            game.has_player(user.id)
                && opponent
                    .and_then(|opponent| (!game.has_player(*opponent)).then(|| ()))
                    .is_none()
        })
        .map(|(index, _)| index)
}

pub async fn start_game<'a>(
    options: &'a [CommandDataOption],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'a, str>> {
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        match a.resolved.as_ref().context("get field `opponent`")? {
            CommandDataOptionValue::User(opponent, _) => Some(opponent),
            x => bail!(
                "In {}/{}: opponent was of incorrect type: Expected User, got {x:?}```",
                file!(),
                line!(),
            ),
        }
    } else {
        None
    };

    let data = ctx.data.read().await;
    let queue = data.get::<Queue>().context("get game queue")?;
    // Without the let it doesn't work for some reason
    #[allow(clippy::let_and_return)]
    let result = if let Some(index) = find_game_index(
        user.id,
        opponent.map(|o| o.id).as_ref(),
        queue.read().await.iter(),
    )
    .await
    {
        let game = queue.write().await.swap_remove(index);
        let mut running_games = data
            .get::<Running>()
            .context("get running games")?
            .write()
            .await;
        running_games.push(game);
        Ok("Game initiated! Use `/ttt set <1-9>` to play!".into())
    } else if let Some(opp) = opponent {
        make_request(opp.clone(), ctx, user).await
    } else {
        Ok("You have no incoming requests".into())
    };
    result
}

pub async fn mark_field<'a>(
    options: &'a [CommandDataOption],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'a, str>> {
    let data = ctx.data.read().await;
    let mut games = data
        .get::<Running>()
        .context("get running games")?
        .write()
        .await;

    let (index, game): (usize, &mut TicTacToe) = if let Some(a) = games
        .iter_mut()
        .enumerate()
        .find(|(_, game)| game.has_player(user.id))
    {
        a
    } else {
        return Ok("You're not in a running game!".into());
    };
    let field_number: usize = options
        .get(0)
        .and_then(|arg| arg.value.as_ref())
        .and_then(serde_json::Value::as_u64)
        .and_then(|x| TryFrom::try_from(x).ok())
        .context("Argument `field` was missing or invalid")?;
    let field_value = if let Some(field) = game.get(field_number.wrapping_sub(1)) {
        *field
    } else {
        return Ok("That is not a valid field!".into());
    };
    if field_value == 0 {
        let player = game.player_number(user.id);
        if game.previous_player == player {
            return Ok("It's not your turn!".into());
        }
        game.insert(player, field_number - 1)
            .context("index out of bounds")?;
        game.previous_player = player;
    }

    let winner = game.check_all();
    if winner == 0 {
        Ok(format!("{:#}", game).into())
    } else {
        let game = games.swap_remove(index);
        Ok(format!("Player {winner} has won!\nPlaying field: \n{game}").into())
    }
}

pub async fn find_game_index(
    opponent: UserId,
    host: Option<&UserId>,
    games: impl Iterator<Item = &TicTacToe>,
) -> Option<usize> {
    for (index, game) in games.enumerate() {
        if game.player_2 == opponent && (host.is_none() || game.player_1 == *host?) {
            return Some(index);
        }
    }
    None
}

#[derive(Clone, PartialEq, Eq)]
pub struct TicTacToe {
    field: [u8; 9],
    pub player_1: UserId,
    pub player_2: UserId,
    pub previous_player: u8,
}

impl TicTacToe {
    pub fn new(player_1: UserId, player_2: UserId) -> Self {
        Self {
            field: [0; 9],
            player_1,
            player_2,
            previous_player: 0,
        }
    }

    pub fn has_player(&self, player: UserId) -> bool {
        player.0 == self.player_1.0 || player.0 == self.player_2.0
    }

    pub fn player_number(&self, player: UserId) -> u8 {
        if player == self.player_1 {
            1
        } else if player == self.player_2 {
            2
        } else {
            0
        }
    }

    pub fn opponent(&self, player: UserId) -> Option<UserId> {
        if player.0 == self.player_1.0 {
            return Some(self.player_2);
        }
        if player.0 == self.player_2.0 {
            return Some(self.player_1);
        }
        None
    }

    pub fn check_rows(&self) -> u8 {
        for i in (0..9).step_by(3) {
            if self.field[i] == self.field[i + 1] && self.field[i] == self.field[i + 2] {
                return self.field[i];
            }
        }
        0
    }

    pub fn check_columns(&self) -> u8 {
        for i in 0..3 {
            if self.field[i] == self.field[i + 3] && self.field[i + 3] == self.field[i + 6] {
                return self.field[i];
            }
        }
        0
    }

    pub fn check_diagonal(&self) -> u8 {
        if self.field[4] != 0
            && ((self.field[0] == self.field[4] && self.field[4] == self.field[8])
                || (self.field[6] == self.field[4] && self.field[4] == self.field[2]))
        {
            self.field[4]
        } else {
            0
        }
    }

    pub fn check_all(&self) -> u8 {
        let columns = self.check_columns();
        if columns != 0 {
            return columns;
        };
        let rows = self.check_rows();
        if rows != 0 {
            return rows;
        }
        self.check_diagonal()
    }

    /// Insert a number into the field.
    /// ## Return
    /// Returns `Some(())` on Success and `None` on Failure
    pub fn insert(&mut self, player: u8, field: usize) -> Option<()> {
        *self.field.get_mut(field)? = player;
        Some(())
    }

    pub fn get(&self, field: usize) -> Option<&u8> {
        self.field.get(field)
    }
}

impl fmt::Display for TicTacToe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            const NUMBER_FIELD: [&str; 9] = [
                ":one:", ":two:", ":three:", ":four:", ":five:", ":six:", ":seven:", ":eight:",
                ":nine:",
            ];
            for (index, (element, nr)) in self.field.iter().zip(NUMBER_FIELD.iter()).enumerate() {
                match element {
                    0 => write!(f, "{nr}")?,
                    1 => write!(f, ":negative_squared_cross_mark:")?,
                    2 => write!(f, ":o2:")?,
                    _ => write!(f, ":interrobang:")?,
                }
                if (index + 1) % 3 == 0 {
                    writeln!(f)?;
                }
            }
        } else {
            for (index, element) in self.field.iter().enumerate() {
                match element {
                    0 => write!(f, ":blue_square:")?,
                    1 => write!(f, ":negative_squared_cross_mark:")?,
                    2 => write!(f, ":o2:")?,
                    _ => write!(f, ":interrobang:")?,
                }
                if (index + 1) % 3 == 0 {
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

pub struct Running;
impl TypeMapKey for Running {
    type Value = RwLock<Vec<TicTacToe>>;
}
pub struct Queue;
impl TypeMapKey for Queue {
    type Value = RwLock<Vec<TicTacToe>>;
}
