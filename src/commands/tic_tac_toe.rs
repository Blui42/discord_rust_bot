#![cfg(feature="tic_tac_toe")]


use serenity::{
    model::{channel::Message, interactions::{self, application_command::{ApplicationCommandOptionType, ApplicationCommandInteractionDataOption}}, id::UserId, prelude:: *},
    prelude::*,
};
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;


pub async fn command(options: &[ApplicationCommandInteractionDataOption], ctx: &Context, user: &User) -> Option<String>{
    match options.get(0)?.name.as_str() {
        "start" => start_game(options.get(0)?.options.as_slice(), ctx, user).await,
        _ => Some("Unknown Subcommand".to_string())
    }
}

pub async fn make_request(opponent: UserId, ctx: &Context, user: &User) -> Option<String>{
    println!("Cool");
    {
        let data = ctx.data.read().await;
        println!("LOCK");
        let current_games = data.get::<TicTacToeRunning>()?;
        println!("CURRENT");
        for game in current_games {
            if let Some(oppnent) = game.opponent(user.id) {
                println!("Schon in nem Spiel");
                return Some(format!("You are already in a game against {}!", oppnent.to_user(&ctx.http).await.ok()?.tag()))
            }
            if game.has_player(opponent){
                return Some(format!("{} is already in a game!", user.tag()))
            }
        }
        let game_queue = data.get::<TicTacToeQueue>()?;
        println!("QUEUE");
        for game in game_queue {
            if game.player_2 == opponent && game.player_1 == user.id {
                return Some(format!("You have already requested a game against {}!", opponent.to_user(&ctx.http).await.ok()?.tag()));
            }
        }
    }
    let mut data = ctx.data.write().await;
    let game_queue = data.get_mut::<TicTacToeQueue>()?;
    game_queue.push(TicTacToe::new(user.id, opponent));
    Some(format!("You challanged {}!", opponent.mention()))

}


pub async fn start_game(options: &[ApplicationCommandInteractionDataOption], ctx: &Context, user: &User) -> Option<String>{
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        if let ApplicationCommandInteractionDataOptionValue::User(opponent, _) = a.resolved.as_ref()? {
            Some(opponent.id)
        }else{return Some(format!("Something went wrong. Report the following in the Support Discord: ```In {}/{}: opponent was of incorrect type, expected User. More Info: \n{:#?}```", file!(), line!(), a.resolved))}
    }else{None};
    println!("{:#?}", opponent);


    let mut data = ctx.data.write().await;
    let game_queue = data.get_mut::<TicTacToeQueue>()?;
    let index = find_game_index(&user.id, opponent.as_ref(),game_queue.as_slice()).await;
    if index.is_none() {
        if let Some(opp) = opponent {
            drop(data);
            return make_request(opp, ctx, user).await;
        }
        else{
            return Some("You have no incoming requests".to_string());
        }
    }
    let game = game_queue.swap_remove(index?);        
    let running_games = data.get_mut::<TicTacToeRunning>()?;
    running_games.push(game);
    return Some(format!("Game initiated! Use `/ttt set <1-9>` to play!"));
}

pub async fn find_game<'a>(host: &UserId, opponent: Option<&UserId>, games: &'a [TicTacToe]) ->  Option<&'a TicTacToe> {
    for game in games {
        if game.player_2 == *host {
            if opponent.is_none()
            || game.player_1 == *opponent? {
                return Some(game)
            } 
        }
    }
    None
}
pub async fn find_game_index(host: &UserId, opponent: Option<&UserId>, games: &[TicTacToe]) ->  Option<usize> {
    for (index, game) in games.iter().enumerate() {
        if game.player_2 == *host {
            if opponent.is_none()
            || game.player_1 == *opponent? {
                return Some(index)
            } 

        }
    }
    None
}

#[derive(Clone)]
pub struct TicTacToe {
    pub field: [[u8;3];3],
    pub player_1: UserId,
    pub player_2: UserId,
}

impl TicTacToe {

    pub const EMPTY: u8 = 0;
    pub const PLAYER_1: u8 = 1;
    pub const PLAYER_2: u8 = 2;

    pub fn new(player_1: UserId, player_2: UserId) -> Self {
        return Self {
            field: [[0;3];3],
            player_1,
            player_2,
        }
    }

    pub fn has_player(&self, player: UserId) -> bool {
        return player.0 == self.player_1.0
            || player.0 == self.player_2.0
    }

    pub fn opponent(&self, player: UserId) -> Option<UserId> {
        if player.0 == self.player_1.0 {
            return Some(self.player_2)
        }
        if player.0 == self.player_2.0 {
            return Some(self.player_1)
        }
        None
    }

    pub fn check_rows(&self) -> u8 {
        for i in 0..3 {
            if self.field[i][0] != 0 {
                if self.field[i].iter().all(|x| *x == 1) {
                    return 1;
                }
                if self.field[i].iter().all(|x| *x == 2) {
                    return 2;
                }
            }
        }
        0
    }

    pub fn check_columns(&self) -> u8 {
        for i in 0..3 {
            if self.field[0][i] != 0 {
                if self.field.iter().all(|x| x[i] == 1) {
                    return 1;
                }
                if self.field.iter().all(|x| x[i] == 2) {
                    return 2;
                }
            }
        }
        0
    }

    pub fn check_diagonal(&self) -> u8 {
        if self.field[1][1] == 0 {
            return 0;
        }
        if (self.field[0][0] == self.field[1][1] && self.field[1][1] == self.field[2][2])
        || (self.field[0][2] == self.field[1][1] && self.field[1][1] == self.field[2][0]){
            return self.field[1][1]
        }
        0
    }

    pub fn check_all(&self) -> u8 {
        let columns = self.check_columns();
        if columns != 0 {return columns};
        let rows = self.check_rows();
        if rows != 0 {return rows}
        return self.check_diagonal();
    }

    pub fn insert(&mut self, player: u8, field: u8) -> Result<(), String>{
        if field > 9 || field == 0 {
            return Err("Not a field".to_string())
        }
        if field <= 3 {
            *self.field[0].get_mut((field-1) as usize).ok_or("Logikfehler lol".to_string())? = player;
            return Ok(());
        }
        if field <= 6 {
            *self.field[1].get_mut((field-4) as usize).ok_or("Logikfehler lol".to_string())? = player;
            return Ok(());
        }

        *self.field[2].get_mut((field-7) as usize).ok_or("Logikfehler lol".to_string())? = player;
        return Ok(());
    }
}

pub struct TicTacToeRunning;
impl TypeMapKey for TicTacToeRunning{

    type Value = Vec<TicTacToe>; 
}
pub struct TicTacToeQueue;
impl TypeMapKey for TicTacToeQueue{

    type Value = Vec<TicTacToe>; 
}