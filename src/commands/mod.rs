pub mod admin;
pub mod fun;
pub mod info;
pub mod rock_paper_scissors;
pub mod tic_tac_toe;

use crate::data::Data;

pub fn commands() -> Vec<poise::Command<Data, anyhow::Error>> {
    vec![
        admin::delete(),
        fun::roll(),
        fun::coin(),
        info::id(),
        info::picture(),
        rock_paper_scissors::rock_paper_scissors(),
        tic_tac_toe::ttt(),
    ]
}
