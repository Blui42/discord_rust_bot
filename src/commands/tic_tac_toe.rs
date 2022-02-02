#![cfg(feature="tic_tac_toe")]



use serenity::{
    model::{
        interactions::application_command::{
            ApplicationCommandInteractionDataOptionValue,
            ApplicationCommandInteractionDataOption
        },
        id::UserId, 
        prelude::*
    },
    prelude::*
};
use tokio::sync::RwLock;


pub async fn command(options: &[ApplicationCommandInteractionDataOption], ctx: &Context, user: &User) -> Option<String>{
    match options.get(0)?.name.as_str() {
        "start" => start_game(options.get(0)?.options.as_slice(), ctx, user).await,
        "set" => mark_field(options.get(0)?.options.as_slice(), ctx, user).await,
        "cancel" => cancel_game(options.get(0)?.options.as_slice(), ctx, user).await,
        _ => Some("Unknown Subcommand".to_string())
    }
}

pub async fn make_request(opponent: UserId, ctx: &Context, user: &User) -> Option<String>{
    let data = ctx.data.read().await;
    let current_games = data.get::<TicTacToeRunning>()?.read().await;
    for game in current_games.iter() {
        if let Some(oppnent) = game.opponent(user.id) {
            println!("Schon in nem Spiel");
            return Some(format!("You are already in a game against {}!", oppnent.to_user(&ctx.http).await.ok()?.tag()))
        }
        if game.has_player(opponent){
            return Some(format!("{} is already in a game!", user.tag()))
        }
    }
    let game_queue = data.get::<TicTacToeQueue>()?.read().await;
    for game in game_queue.iter() {
        if game.player_2 == opponent && game.player_1 == user.id {
            return Some(format!("You have already requested a game against {}!", opponent.to_user(&ctx.http).await.ok()?.tag()));
        }
    }
    let mut game_queue = data.get::<TicTacToeQueue>()?.write().await;
    game_queue.push(TicTacToe::new(user.id, opponent));
    Some(format!("You challanged {}!", opponent.mention()))

}

pub async fn cancel_game(options: &[ApplicationCommandInteractionDataOption], ctx: &Context, user: &User) -> Option<String>{
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        if let ApplicationCommandInteractionDataOptionValue::User(opponent, _) = a.resolved.as_ref()? {
            Some(&opponent.id)
        }else{return Some(format!("Something went wrong. Report the following in the Support Discord: ```In {}/{}: opponent was of incorrect type, expected User. More Info: \n{:#?}```", file!(), line!(), a.resolved))}
    }else{None};

    let data = ctx.data.read().await;
    let game_queue = data.get::<TicTacToeQueue>()?.read().await;
    if let Some(index) = find_game_index2(&user.id, opponent, game_queue.as_slice()).await {
        data.get::<TicTacToeQueue>()?.write().await.swap_remove(index);
        return Some("Cancelled game.".to_string())
    }
    
    
    let running_games = data.get::<TicTacToeRunning>()?.read().await;
    if let Some(index) = find_game_index2(&user.id, opponent, running_games.as_slice()).await {
        data.get::<TicTacToeRunning>()?.write().await.swap_remove(index);
        return Some("You gave up.".to_string())
    }
    return Some("There was no game to cancel".to_string());
    
}

pub async fn start_game(options: &[ApplicationCommandInteractionDataOption], ctx: &Context, user: &User) -> Option<String>{
    // This will be Some(opponent) if the user spefified an opponent, otherwise None.
    let opponent = if let Some(a) = options.get(0) {
        if let ApplicationCommandInteractionDataOptionValue::User(opponent, _) = a.resolved.as_ref()? {
            Some(opponent.id)
        }else{return Some(format!("Something went wrong. Report the following in the Support Discord: ```In {}/{}: opponent was of incorrect type, expected User. More Info: \n{:#?}```", file!(), line!(), a.resolved))}
    }else{None};
    println!("{:#?}", opponent);


    let data = ctx.data.read().await;
    let mut game_queue = data.get::<TicTacToeQueue>()?.write().await;
    let index = find_game_index(&user.id, opponent.as_ref(),game_queue.as_slice()).await;
    if index.is_none() {
        if let Some(opp) = opponent {
            return make_request(opp, ctx, user).await;
        }
        else{
            return Some("You have no incoming requests".to_string());
        }
    }
    let game = game_queue.swap_remove(index?);        
    let mut running_games = data.get::<TicTacToeRunning>()?.write().await;
    running_games.push(game);
    return Some(format!("Game initiated! Use `/ttt set <1-9>` to play!"));
}

pub async fn mark_field(options: &[ApplicationCommandInteractionDataOption], ctx: &Context, user: &User) -> Option<String>{
    let data = ctx.data.read().await;
    let mut games = data.get::<TicTacToeRunning>()?.write().await;
    let mut index = 0;
    let mut game = None;
    for (i, _game) in games.iter_mut().enumerate() {
        if _game.has_player(user.id) {
            game = Some(_game);
            index = i;
            break;
        }
    }
    if game.is_none() {
        return Some("You don't have a running game!".to_string())
    }
    let game = game?;
    let field_number: u8 = if let Some(a) = validate_field_number(options.get(0)?.value.as_ref()?.as_i64()?) {
        a
    } else {
        return Some("That's an invalid field number!".to_string())
    };
    if game.get(field_number)? == 0 {
        let player = if game.player_1 == user.id {1} else {2};
        if game.previous_player == player {
            return Some("It's not your turn!".to_string())
        }
        game.insert(player, field_number).ok()?;
        game.previous_player = player;
    }
    
    let winner = game.check_all();
    if winner == 0 {
        Some(format!("{}", game.formatted_field()))

    }else{
        let game = games.swap_remove(index);
        Some(format!("Player {} has won!\nPlaying field: \n{}", winner, game.formatted_field()))
    }
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
pub async fn find_game_index(opponent: &UserId, host: Option<&UserId>, games: &[TicTacToe]) ->  Option<usize> {
    for (index, game) in games.iter().enumerate() {
        if game.player_2 == *opponent {
            if host.is_none()
            || game.player_1 == *host? {
                return Some(index)
            } 

        }
    }
    None
}

pub async fn find_game_index2(host: &UserId, opponent: Option<&UserId>, games: &[TicTacToe]) ->  Option<usize> {
    for (index, game) in games.iter().enumerate() {
        if game.player_1 == *host {
            if opponent.is_none()
            || game.player_2 == *opponent? {
                return Some(index)
            } 

        }
    }
    None
}

pub fn validate_field_number(field: i64) -> Option<u8> {
    if field > 0 && field <= 9 {
        Some(field as u8)
    }else{
        None
    }
}

#[derive(Clone, PartialEq)]
pub struct TicTacToe {
    pub field: [[u8;3];3],
    pub player_1: UserId,
    pub player_2: UserId,
    pub previous_player: u8,
}

impl TicTacToe {

    pub fn new(player_1: UserId, player_2: UserId) -> Self {
        return Self {
            field: [[0;3];3],
            player_1,
            player_2,
            previous_player: 0,
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

    pub fn get(&mut self, field: u8) -> Option<u8>{
        if field > 9 || field == 0 {
            return None
        }
        if field <= 3 {
            return Some(*self.field[0].get((field-1) as usize)?);
        }
        if field <= 6 {
            return Some(*self.field[1].get((field-4) as usize)?);
        }
        return Some(*self.field[2].get((field-7) as usize)?);
    }

    pub fn formatted_field(&self) -> String {
        const NUMBER_FIELD: [[&str;3];3] = [
            [":one:", ":two:", ":three:"],
            [":four:", ":five:", ":six:"],
            [":seven:", ":eight:", ":nine:"]
        ];
        let mut ret = String::new();
        for (row, nr) in self.field.iter().zip(NUMBER_FIELD.iter()){
            for (element, nr) in row.iter().zip(nr.iter()) {
                match element {
                    0 => ret += nr,
                    1 => ret += ":negative_squared_cross_mark:",
                    2 => ret += ":o2:",
                    _ => ret += ":interrobang:"
                }
            }
            ret += "\n"
        }
        ret
    }
}

pub struct TicTacToeRunning;
impl TypeMapKey for TicTacToeRunning{

    type Value = RwLock<Vec<TicTacToe>>; 
}
pub struct TicTacToeQueue;
impl TypeMapKey for TicTacToeQueue{

    type Value = RwLock<Vec<TicTacToe>>; 
}