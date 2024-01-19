pub mod config;

use std::collections::HashMap;

use poise::serenity_prelude::UserId;
use tokio::{
    sync::{Mutex, RwLock},
    time::Instant,
};

use crate::commands::{rock_paper_scissors::Rps, tic_tac_toe::TicTacToe};

#[derive(Default, Debug)]
pub struct Data {
    pub rps: Mutex<HashMap<(UserId, UserId), (Rps, Instant)>>,
    pub ttt_games: RwLock<Vec<TicTacToe>>,
    pub ttt_queue: RwLock<HashMap<UserId, UserId>>,
}
