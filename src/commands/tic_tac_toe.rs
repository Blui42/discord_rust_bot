use std::{collections::HashMap, fmt};

use crate::utils::Context;

use poise::{
    serenity_prelude::{self as serenity, CreateAllowedMentions, Mentionable, User, UserId},
    CreateReply,
};

#[poise::command(slash_command, subcommands("start", "set", "cancel"))]
#[allow(clippy::unused_async)]
pub async fn ttt(_: Context<'_>) -> anyhow::Result<()> {
    Ok(())
}
#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>, opponent: Option<serenity::User>) -> anyhow::Result<()> {
    let user = ctx.author();
    let games = ctx.data().ttt_queue.read().await;
    if let Some(&opponent) = find_game(user, opponent.as_ref().map(|o| &o.id), &games) {
        drop(games);
        ctx.data().ttt_queue.write().await.remove(&opponent);
        ctx.data().ttt_games.write().await.push(TicTacToe::new(user.id, opponent));
        ctx.reply("Game initiated! Use `/ttt set <1-9>` to play!").await?;
    } else if let Some(opp) = opponent {
        drop(games);
        make_request(ctx, opp).await?;
    } else {
        ctx.reply("You have no incoming requests").await?;
    }
    Ok(())
}
pub async fn make_request(ctx: Context<'_>, opponent: serenity::User) -> anyhow::Result<()> {
    let user = ctx.author();
    if opponent.id == user.id {
        ctx.reply("You can't play against yourself").await?;
        return Ok(());
    }
    {
        let current_games = ctx.data().ttt_games.read().await;
        for game in &*current_games {
            if let Some(opp) = game.opponent(user.id) {
                ctx.send(
                    CreateReply::default()
                        .allowed_mentions(CreateAllowedMentions::new())
                        .content(format!("You are already in a game against {}!", opp.mention())),
                )
                .await?;
                return Ok(());
            }
            if game.has_player(opponent.id) {
                ctx.reply(format!("{} is already in a game!", opponent.tag())).await?;
                return Ok(());
            }
        }
    }
    let opponent_mentioned = opponent.mention();
    if let Some(previous_opponent) = ctx.data().ttt_queue.write().await.insert(user.id, opponent.id)
    {
        ctx.reply(format!(
            "You cancelled your game against {} and challenged {opponent_mentioned}",
            previous_opponent.to_user(&ctx).await?.tag()
        ))
        .await?
    } else {
        ctx.reply(format!("You challenged {opponent_mentioned}!")).await?
    };

    Ok(())
}

#[poise::command(slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[min = 1]
    #[max = 9]
    field: usize,
) -> anyhow::Result<()> {
    let mut games = ctx.data().ttt_games.write().await;
    let user = ctx.author();

    let Some((index, game)) =
        games.iter_mut().enumerate().find(|(_, game)| game.has_player(user.id))
    else {
        ctx.reply("You're not in a running game!").await?;
        return Ok(());
    };
    let field_index = field.wrapping_sub(1);

    let player = game.player_number(user.id);
    let res = game.place(player, field_index);
    if let Err(err) = res {
        ctx.reply(format!("{err}\n{game:#}")).await?;
        return Ok(());
    }

    if let Some(winner) = game.check_all() {
        let game = games.swap_remove(index);
        drop(games);
        let winner_name = game.player_id(winner).mention();
        ctx.reply(format!("{winner_name} has won!\nPlaying field: \n{game}")).await?;
    } else if game.will_tie() {
        let game = games.swap_remove(index);
        drop(games);
        ctx.reply(format!("It's a tie!\nPlaying field: \n{game}")).await?;
    } else {
        ctx.reply(format!("{game:#}")).await?;
    }
    Ok(())
}
#[poise::command(slash_command)]
pub async fn cancel(ctx: Context<'_>, opponent: Option<serenity::UserId>) -> anyhow::Result<()> {
    let user = ctx.author().id;
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    {
        let mut game_queue = ctx.data().ttt_queue.write().await;
        if game_queue.contains_key(&user) {
            game_queue.remove(&user);
            drop(game_queue);
            ctx.reply("Cancelled game.").await?;
            return Ok(());
        }
    }

    {
        let mut running_games = ctx.data().ttt_games.write().await;
        if let Some(index) = find_game_with_either(&*running_games, user, opponent.as_ref()) {
            running_games.swap_remove(index);
            drop(running_games);
            ctx.reply("You gave up.").await?;
            return Ok(());
        }
    }
    ctx.reply("There was no game to cancel").await?;
    Ok(())
}

fn find_game_with_either<'a>(
    game_queue: impl IntoIterator<Item = &'a TicTacToe>,
    user: UserId,
    opponent: Option<&UserId>,
) -> Option<usize> {
    game_queue
        .into_iter()
        .enumerate()
        .find(|(_, game)| {
            game.has_player(user) && opponent.map_or(true, |opp| game.has_player(*opp))
        })
        .map(|(index, _)| index)
}

pub fn find_game<'a>(
    opponent: &'_ User,
    host: Option<&'a UserId>,
    games: &'a HashMap<UserId, UserId>,
) -> Option<&'a UserId> {
    if let Some(host) = host {
        return (games.get(host) == Some(&opponent.id)).then_some(host);
    }
    games.iter().find(|(_, o)| **o == opponent.id).map(|h| h.0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicTacToe {
    field: [Option<Player>; 9],
    pub player_1: UserId,
    pub player_2: UserId,
    pub previous_player: Option<Player>,
}

impl TicTacToe {
    pub const fn new(player_1: UserId, player_2: UserId) -> Self {
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

    pub const fn player_id(&self, player: Player) -> UserId {
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
        for f in &mut game.field {
            if f.is_none() {
                *f = Some(Player::One);
            }
        }

        if game.check_all().is_some() {
            return false;
        }

        game.field = self.field;
        for f in &mut game.field {
            if f.is_none() {
                *f = Some(Player::Two);
            }
        }
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
            for (index, (element, nr)) in self.field.iter().zip(&NUMBER_FIELD).enumerate() {
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
            Self::AlreadyFull => write!(f, "That field was already used!"),
            Self::OutOfBounds => write!(f, "That's not a valid field!"),
            Self::NotYourTurn => write!(f, "It's not your turn!"),
        }
    }
}
