use std::{collections::HashMap, fmt};

use crate::utils::{get_data, CommandResult};

use anyhow::{bail, Context as _, Result};
use serenity::all::{Context, Mentionable, ResolvedOption, ResolvedValue, User, UserId};
use serenity::prelude::TypeMapKey;
use tokio::sync::RwLock;

pub async fn command<'a>(
    options: &'a [ResolvedOption<'a>],
    ctx: &Context,
    user: &User,
) -> CommandResult {
    let subcommand = options.get(0).context("get subcommand")?;
    let ResolvedValue::SubCommand(subcommand_options) = &subcommand.value else {
        bail!("Missing Subcommand Arguments");
    };
    match subcommand.name {
        "start" => start_game(subcommand_options.as_slice(), ctx, user).await,
        "set" => mark_field(subcommand_options.as_slice(), ctx, user).await,
        "cancel" => cancel_game(subcommand_options.as_slice(), ctx, user).await,
        _ => bail!("Unknown Command: {subcommand:?}"),
    }
}

pub async fn make_request(opponent: User, ctx: &Context, user: &User) -> CommandResult {
    if opponent.id == user.id {
        return Ok("That would be kind of sad".into());
    }
    let data = ctx.data.read().await;
    let current_games = get_data::<Running>(&data)?.read().await;
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
    let mut game_queue = get_data::<Queue>(&data)?.write().await;
    let opponent_mentioned = opponent.mention();
    match game_queue.insert(user.id, opponent.id) {
        Some(previous_opponent) => Ok(format!(
            "You cancelled your game against {} and challenged {opponent_mentioned}",
            previous_opponent.to_user(&ctx.http).await?.tag()
        )
        .into()),
        None => Ok(format!("You challenged {opponent_mentioned}!").into()),
    }
}

pub async fn cancel_game<'a>(
    options: &'a [ResolvedOption<'_>],
    ctx: &Context,
    user: &User,
) -> CommandResult {
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        match &a.value {
            ResolvedValue::User(opponent, _) => Some(&opponent.id),
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
    let game_queue = get_data::<Queue>(&data)?.read().await;
    if game_queue.contains_key(&user.id) {
        drop(game_queue);
        get_data::<Queue>(&data)?.write().await.remove(&user.id);
        return Ok("Cancelled game.".into());
    }

    let running_games = get_data::<Running>(&data)?.read().await;
    if let Some(index) = find_game_with_either(running_games.iter(), user, opponent) {
        drop(running_games);
        get_data::<Running>(&data)?.write().await.swap_remove(index);
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
            game.has_player(user.id) && opponent.map_or(true, |opp| game.has_player(*opp))
        })
        .map(|(index, _)| index)
}

pub async fn start_game(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    user: &User,
) -> CommandResult {
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        match &a.value {
            ResolvedValue::User(opponent, _) => Some(*opponent),
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
    let queue = get_data::<Queue>(&data)?;
    let games = queue.read().await;
    if let Some(&opponent) = find_game(user, opponent.map(|o| &o.id), &games).await {
        drop(games);
        queue.write().await.remove(&opponent);
        let mut running_games = get_data::<Running>(&data)?.write().await;
        running_games.push(TicTacToe::new(user.id, opponent));
        Ok("Game initiated! Use `/ttt set <1-9>` to play!".into())
    } else if let Some(opp) = opponent {
        drop(games);
        make_request(opp.clone(), ctx, user).await
    } else {
        Ok("You have no incoming requests".into())
    }
}

pub async fn mark_field<'a>(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    user: &User,
) -> CommandResult {
    let data = ctx.data.read().await;
    let mut games = get_data::<Running>(&data)?.write().await;

    let (index, game): (usize, &mut TicTacToe) =
        match games.iter_mut().enumerate().find(|(_, game)| game.has_player(user.id)) {
            Some(a) => a,
            None => return Ok("You're not in a running game!".into()),
        };
    let field_number = match options.first().map(|o| &o.value) {
        Some(ResolvedValue::Integer(x)) => usize::try_from(*x),
        _ => bail!("Argument `field`: expected Integer, got {:?}", options.first()),
    }?;

    let player = game.player_number(user.id);
    let res = game.place(player, field_number.wrapping_sub(1));
    match res {
        Ok(()) => (),
        Err(PlaceError::NotYourTurn) => return Ok("It's not your turn!".into()),
        Err(PlaceError::OutOfBounds) => anyhow::bail!("Field was not in range 1-9"),
        Err(PlaceError::AlreadyFull) => return Ok(format!("Sorry but no.\n{game:#}").into()),
    }

    if let Some(winner) = game.check_all() {
        let game = games.swap_remove(index);
        let winner_name = game.player_id(winner).mention();
        Ok(format!("{winner_name} has won!\nPlaying field: \n{game}").into())
    } else if game.will_tie() {
        let game = games.swap_remove(index);
        Ok(format!("It's a tie!\nPlaying field: \n{game}").into())
    } else {
        Ok(format!("{game:#}").into())
    }
}

pub async fn find_game<'a>(
    opponent: &'_ User,
    host: Option<&'a UserId>,
    games: &'a HashMap<UserId, UserId>,
) -> Option<&'a UserId> {
    if let Some(host) = host {
        return (games.get(host) == Some(&opponent.id)).then_some(host);
    }
    games.iter().find(|(_, o)| **o == opponent.id).map(|h| h.0)
}

#[derive(Clone, PartialEq, Eq)]
pub struct TicTacToe {
    field: [Option<Player>; 9],
    pub player_1: UserId,
    pub player_2: UserId,
    pub previous_player: Option<Player>,
}

impl TicTacToe {
    pub fn new(player_1: UserId, player_2: UserId) -> Self {
        Self { field: [None; 9], player_1, player_2, previous_player: None }
    }

    pub fn has_player(&self, player: UserId) -> bool {
        player == self.player_1 || player == self.player_2
    }

    pub fn player_number(&self, player: UserId) -> Option<Player> {
        if player == self.player_1 {
            Some(Player::One)
        } else if player == self.player_2 {
            Some(Player::Two)
        } else {
            None
        }
    }

    pub fn player_id(&self, player: Player) -> UserId {
        match player {
            Player::One => self.player_1,
            Player::Two => self.player_2,
        }
    }

    pub fn opponent(&self, player: UserId) -> Option<UserId> {
        if player == self.player_1 {
            return Some(self.player_2);
        }
        if player == self.player_2 {
            return Some(self.player_1);
        }
        None
    }

    pub fn check_rows(&self) -> Option<Player> {
        self.field
            .chunks_exact(3)
            .find(|x| x[0].is_some() && x[0] == x[1] && x[1] == x[2])
            .and_then(|x| x[0])
    }

    pub fn check_columns(&self) -> Option<Player> {
        for i in 0..3 {
            if self.field[i].is_some()
                && self.field[i] == self.field[i + 3]
                && self.field[i + 3] == self.field[i + 6]
            {
                return self.field[i];
            }
        }
        None
    }

    pub fn check_diagonal(&self) -> Option<Player> {
        if self.field[4].is_some()
            && ((self.field[0] == self.field[4] && self.field[4] == self.field[8])
                || (self.field[6] == self.field[4] && self.field[4] == self.field[2]))
        {
            self.field[4]
        } else {
            None
        }
    }

    pub fn will_tie(&self) -> bool {
        if self.is_full() {
            return true;
        }

        let mut game = self.clone();
        game.field.iter_mut().for_each(|f| {
            if f.is_none() {
                *f = Some(Player::One);
            }
        });

        if game.check_all().is_some() {
            return false;
        }

        game.field = self.field;
        game.field.iter_mut().for_each(|f| {
            if f.is_none() {
                *f = Some(Player::Two);
            }
        });
        if game.check_all().is_some() {
            return false;
        }
        true
    }

    pub fn is_full(&self) -> bool {
        self.field.iter().all(Option::is_some)
    }

    pub fn check_all(&self) -> Option<Player> {
        self.check_columns().or_else(|| self.check_rows()).or_else(|| self.check_diagonal())
    }

    pub fn place(&mut self, player: Option<Player>, field: usize) -> Result<(), PlaceError> {
        use PlaceError::{AlreadyFull, NotYourTurn, OutOfBounds};
        if self.previous_player == player {
            return Err(NotYourTurn);
        }
        let field = self.field.get_mut(field).ok_or(OutOfBounds)?;

        if field.is_some() {
            return Err(AlreadyFull);
        }

        *field = player;
        Ok(())
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
                    None => write!(f, "{nr}")?,
                    Some(Player::One) => write!(f, ":negative_squared_cross_mark:")?,
                    Some(Player::Two) => write!(f, ":o2:")?,
                }
                if (index + 1) % 3 == 0 {
                    writeln!(f)?;
                }
            }
        } else {
            for (index, element) in self.field.iter().enumerate() {
                match element {
                    None => write!(f, ":blue_square:")?,
                    Some(Player::One) => write!(f, ":negative_squared_cross_mark:")?,
                    Some(Player::Two) => write!(f, ":o2:")?,
                }
                if (index + 1) % 3 == 0 {
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    One,
    Two,
}
impl TryFrom<u8> for Player {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(()),
        }
    }
}

pub enum PlaceError {
    AlreadyFull,
    OutOfBounds,
    NotYourTurn,
}
impl fmt::Display for PlaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaceError::AlreadyFull => write!(f, "That field was already used!"),
            PlaceError::OutOfBounds => write!(f, "That's not a valid field!"),
            PlaceError::NotYourTurn => write!(f, "It's not your turn!"),
        }
    }
}

pub struct Running;
impl TypeMapKey for Running {
    type Value = RwLock<Vec<TicTacToe>>;
}
pub struct Queue;
impl TypeMapKey for Queue {
    type Value = RwLock<HashMap<UserId, UserId>>;
}
