use crate::{lex::lex, rule::parse, world::Grid};

mod ascii;
mod lex;
mod rule;
mod unit;
mod world;

fn main() {
    let grid = Grid::from_ascii("BA IS YO BA IS KE");
    println!("'{}'\n============", grid.to_ascii());
    for (i, rule) in parse(&lex(&grid)).iter().enumerate() {
        println!("{} | {}", i + 1, rule.to_ascii());
    }
}
