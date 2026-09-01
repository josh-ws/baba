use crate::view::run_game;

mod ascii;
mod eval;
mod level;
mod lex;
mod rule;
mod unit;
mod view;
mod world;

#[macroquad::main("baba")]
async fn main() {
    let data = include_str!("../assets/levels/where.txt");
    run_game(data).await
}
