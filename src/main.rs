use crate::{pack::Levelpack, view::run_game};

mod ascii;
mod eval;
mod level;
mod lex;
mod pack;
mod rule;
mod unit;
mod view;
mod world;

#[macroquad::main("baba")]
async fn main() {
    let mut pack = Levelpack::parse(include_str!("../assets/packs/demo.txt"));
    let level = pack.get_level("map");
    run_game(level).await
}
