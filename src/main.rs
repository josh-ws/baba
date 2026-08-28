use crate::{
    eval::Turn,
    world::{Direction, Grid},
};

mod ascii;
mod eval;
mod lex;
mod rule;
mod unit;
mod world;

fn main() {
    let mut grid = Grid::from_ascii("BA IS YO .. .. ba .. ..");
    let mut new_turn = Turn::new(&mut grid, Direction::East);
    new_turn.run();
    println!("{}", grid.to_ascii());
}
