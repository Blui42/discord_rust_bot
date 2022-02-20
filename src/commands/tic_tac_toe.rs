#![cfg(feature = "tic_tac_toe")]

use serenity::{
    model::{
        id::UserId,
        interactions::application_command::{
            ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue,
        },
        prelude::*,
    },
    prelude::*,
};
use std::fmt;
use tokio::sync::RwLock;

pub async fn command(
    options: &[ApplicationCommandInteractionDataOption],
    ctx: &Context,
    user: &User,
) -> Option<String> {
    match options.get(0)?.name.as_str() {
        "start" => start_game(options.get(0)?.options.as_slice(), ctx, user).await,
        "set" => mark_field(options.get(0)?.options.as_slice(), ctx, user).await,
        "cancel" => cancel_game(options.get(0)?.options.as_slice(), ctx, user).await,
        _ => Some("Unknown Subcommand".to_string()),
    }
}

pub async fn make_request(opponent: UserId, ctx: &Context, user: &User) -> Option<String> {
    if opponent == user.id {
        return Some("That would be kind of sad".to_string());
    }
    let data = ctx.data.read().await;
    let current_games = data.get::<TicTacToeRunning>()?.read().await;
    for game in current_games.iter() {
        if let Some(oppnent) = game.opponent(user.id) {
            return Some(format!(
                "You are already in a game against {}!",
                oppnent.to_user(&ctx.http).await.ok()?.tag()
            ));
        }
        if game.has_player(opponent) {
            return Some(format!("{} is already in a game!", user.tag()));
        }
    }
    let game_queue = data.get::<TicTacToeQueue>()?.read().await;
    for game in game_queue.iter() {
        if game.player_2 == opponent && game.player_1 == user.id {
            return Some(format!(
                "You have already requested a game against {}!",
                opponent.to_user(&ctx.http).await.ok()?.tag()
            ));
        }
    }
    let mut game_queue = data.get::<TicTacToeQueue>()?.write().await;
    game_queue.push(TicTacToe::new(user.id, opponent));
    Some(format!("You challanged {}!", opponent.mention()))
}

pub async fn cancel_game(
    options: &[ApplicationCommandInteractionDataOption],
    ctx: &Context,
    user: &User,
) -> Option<String> {
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        if let ApplicationCommandInteractionDataOptionValue::User(opponent, _) =
            a.resolved.as_ref()?
        {
            Some(&opponent.id)
        } else {
            return Some(format!("Something went wrong. Report the following in the Support Discord: ```In {}/{}: opponent was of incorrect type, expected User. More Info: \n{:#?}```", file!(), line!(), a.resolved));
        }
    } else {
        None
    };

    let data = ctx.data.read().await;
    let game_queue = data.get::<TicTacToeQueue>()?.read().await;
    if let Some(index) = find_game_index2(&user.id, opponent, game_queue.as_slice()).await {
        data.get::<TicTacToeQueue>()?
            .write()
            .await
            .swap_remove(index);
        return Some("Cancelled game.".to_string());
    }

    let running_games = data.get::<TicTacToeRunning>()?.read().await;
    if let Some(index) = find_game_index2(&user.id, opponent, running_games.as_slice()).await {
        data.get::<TicTacToeRunning>()?
            .write()
            .await
            .swap_remove(index);
        return Some("You gave up.".to_string());
    }
    Some("There was no game to cancel".to_string())
}

pub async fn start_game(
    options: &[ApplicationCommandInteractionDataOption],
    ctx: &Context,
    user: &User,
) -> Option<String> {
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        if let ApplicationCommandInteractionDataOptionValue::User(opponent, _) =
            a.resolved.as_ref()?
        {
            Some(opponent.id)
        } else {
            return Some(format!("Something went wrong. Report the following in the Support Discord: ```In {}/{}: opponent was of incorrect type, expected User. More Info: \n{:#?}```", file!(), line!(), a.resolved));
        }
    } else {
        None
    };
    println!("{:#?}", opponent);

    let data = ctx.data.read().await;
    let mut game_queue = data.get::<TicTacToeQueue>()?.write().await;
    if let Some(index) = find_game_index(&user.id, opponent.as_ref(), game_queue.as_slice()).await {
        let game = game_queue.swap_remove(index);
        let mut running_games = data.get::<TicTacToeRunning>()?.write().await;
        running_games.push(game);
        Some("Game initiated! Use `/ttt set <1-9>` to play!".to_string())
    } else if let Some(opp) = opponent {
        make_request(opp, ctx, user).await
    } else {
        Some("You have no incoming requests".to_string())
    }
}

pub async fn mark_field(
    options: &[ApplicationCommandInteractionDataOption],
    ctx: &Context,
    user: &User,
) -> Option<String> {
    let data = ctx.data.read().await;
    let mut games = data.get::<TicTacToeRunning>()?.write().await;

    let (index, game) = if let Some(a) = games
        .iter_mut()
        .enumerate()
        .find(|(_, game)| game.has_player(user.id))
    {
        a
    } else {
        return Some("You don't have a running game!".to_string());
    };
    let field_number = options.get(0)?.value.as_ref()?.as_u64()? as usize;
    if field_number == 0 || field_number > 9 {
        return Some("That is not a valid field!".to_string());
    }
    if *game.get(field_number)? == 0 {
        let player = game.player_number(user.id);
        if game.previous_player == player {
            return Some("It's not your turn!".to_string());
        }
        game.insert(player, field_number as usize)?;
        game.previous_player = player;
    }

    let winner = game.check_all();
    if winner == 0 {
        Some(format!("{:#}", game))
    } else {
        let game = games.swap_remove(index);
        Some(format!(
            "Player {} has won!\nPlaying field: \n{}",
            winner, game
        ))
    }
}

pub async fn find_game_index(
    opponent: &UserId,
    host: Option<&UserId>,
    games: &[TicTacToe],
) -> Option<usize> {
    for (index, game) in games.iter().enumerate() {
        if game.player_2 == *opponent && (host.is_none() || game.player_1 == *host?) {
            return Some(index);
        }
    }
    None
}

pub async fn find_game_index2(
    host: &UserId,
    opponent: Option<&UserId>,
    games: &[TicTacToe],
) -> Option<usize> {
    for (index, game) in games.iter().enumerate() {
        if game.player_1 == *host && (opponent.is_none() || game.player_2 == *opponent?) {
            return Some(index);
        }
    }
    None
}

#[derive(Clone, PartialEq)]
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
        if self.field[4] == 0 {
            0
        } else if (self.field[0] == self.field[4] && self.field[4] == self.field[8])
            || (self.field[6] == self.field[4] && self.field[4] == self.field[2])
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
                    0 => write!(f, "{}", nr)?,
                    1 => write!(f, ":negative_squared_cross_mark:")?,
                    2 => write!(f, ":o2:")?,
                    _ => write!(f, ":interrobang:")?,
                }
                if (index + 1) % 3 == 0 {
                    writeln!(f)?
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
                    writeln!(f)?
                }
            }
        }
        Ok(())
    }
}

pub struct TicTacToeRunning;
impl TypeMapKey for TicTacToeRunning {
    type Value = RwLock<Vec<TicTacToe>>;
}
pub struct TicTacToeQueue;
impl TypeMapKey for TicTacToeQueue {
    type Value = RwLock<Vec<TicTacToe>>;
}
