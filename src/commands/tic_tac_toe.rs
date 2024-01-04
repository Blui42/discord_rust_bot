use anyhow::{bail, Context as _, Result};
use serenity::{
    client::Context,
    model::{
        application::{ResolvedOption, ResolvedValue},
        id::UserId,
        mention::Mentionable,
        user::User,
    },
    prelude::TypeMapKey,
};
use std::{borrow::Cow, collections::HashMap, fmt};
use tokio::sync::RwLock;

pub async fn command<'a>(
    options: &'a [ResolvedOption<'a>],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'static, str>> {
    let subcommand = options.get(0).context("get subcommand")?;
    let ResolvedValue::SubCommand(subcommand_options) = &subcommand.value else {
        bail!("Missing Subcommand Arguments");
    };
    match subcommand.name {
        "start" => start_game(subcommand_options.as_slice(), ctx, user).await,
        "set" => mark_field(subcommand_options.as_slice(), ctx, user).await,
        "cancel" => cancel_game(subcommand_options.as_slice(), ctx, user).await,
        _ => Ok("Unknown Subcommand".into()),
    }
}

pub async fn make_request(opponent: User, ctx: &Context, user: &User) -> Result<Cow<'static, str>> {
    if opponent.id == user.id {
        return Ok("That would be kind of sad".into());
    }
    let data = ctx.data.read().await;
    let current_games = data.get::<Running>().context("get running games")?.read().await;
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
    let mut game_queue = data.get::<Queue>().context("Get game queue")?.write().await;
    let opponent_mentioned = opponent.mention();
    match game_queue.insert(user.clone(), opponent) {
        Some(previous_opponent) => Ok(format!(
            "You cancelled your game against {} and challenged {opponent_mentioned}",
            previous_opponent.tag()
        )
        .into()),
        None => Ok(format!("You challenged {opponent_mentioned}!").into()),
    }
}

pub async fn cancel_game<'a>(
    options: &'a [ResolvedOption<'_>],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'static, str>> {
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
    let game_queue = data.get::<Queue>().context("get game queue")?.read().await;
    if game_queue.contains_key(user) {
        drop(game_queue);
        data.get::<Queue>().context("get game queue")?.write().await.remove(user);
        return Ok("Cancelled game.".into());
    }

    let running_games = data.get::<Running>().context("get running games")?.read().await;
    if let Some(index) = find_game_with_either(running_games.iter(), user, opponent) {
        drop(running_games);
        data.get::<Running>().context("get running games")?.write().await.swap_remove(index);
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

pub async fn start_game<'a>(
    options: &'a [ResolvedOption<'a>],
    ctx: &Context,
    user: &User,
) -> Result<Cow<'static, str>> {
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
    let queue = data.get::<Queue>().context("get game queue")?;
    let games = queue.read().await;
    if let Some(opponent) = find_game(user, opponent, &games).await {
        let opponent = opponent.clone();
        drop(games);
        queue.write().await.remove(&opponent);
        let mut running_games = data.get::<Running>().context("get running games")?.write().await;
        running_games.push(TicTacToe::new(user.id, opponent.id));
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
) -> Result<Cow<'static, str>> {
    let data = ctx.data.read().await;
    let mut games = data.get::<Running>().context("get running games")?.write().await;

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

    let winner = game.check_all();
    if winner != 0 {
        let game = games.swap_remove(index);
        Ok(format!("Player {winner} has won!\nPlaying field: \n{game}").into())
    } else if game.will_tie() {
        let game = games.swap_remove(index);
        Ok(format!("It's a tie!\nPlaying field: \n{game}").into())
    } else {
        Ok(format!("{game:#}").into())
    }
}

pub async fn find_game<'a>(
    opponent: &'_ User,
    host: Option<&'a User>,
    games: &'a HashMap<User, User>,
) -> Option<&'a User> {
    if let Some(host) = host {
        return (games.get(host) == Some(opponent)).then_some(host);
    }
    games.iter().find(|(_, o)| **o == *opponent).map(|h| h.0)
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
        Self { field: [0; 9], player_1, player_2, previous_player: 0 }
    }

    pub fn has_player(&self, player: UserId) -> bool {
        player == self.player_1 || player == self.player_2
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
        if player == self.player_1 {
            return Some(self.player_2);
        }
        if player == self.player_2 {
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

    pub fn will_tie(&self) -> bool {
        if self.is_full() {
            return true;
        }

        let mut game = self.clone();
        game.field.iter_mut().for_each(|f| {
            if *f == 0 {
                *f = 1;
            }
        });

        if game.check_all() != 0 {
            return false;
        }

        game.field = self.field;
        game.field.iter_mut().for_each(|f| {
            if *f == 0 {
                *f = 2;
            }
        });
        if game.check_all() != 0 {
            return false;
        }
        true
    }

    pub fn is_full(&self) -> bool {
        self.field.iter().all(|square| *square != 0)
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

    pub fn place(&mut self, player: u8, field: usize) -> Result<(), PlaceError> {
        use PlaceError::{AlreadyFull, NotYourTurn, OutOfBounds};
        if self.previous_player == player {
            return Err(NotYourTurn);
        }
        let field = self.field.get_mut(field).ok_or(OutOfBounds)?;

        if *field != 0 {
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
    type Value = RwLock<HashMap<User, User>>;
}
