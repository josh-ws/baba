use std::io::{self, BufRead, Write, stdout};

use crate::{
    eval::{Turn, TurnResult},
    world::{Direction, Grid},
};

mod ascii;
mod eval;
mod lex;
mod rule;
mod unit;
mod world;

fn match_direction(c: Option<char>) -> Option<Direction> {
    match c {
        Some(c) => match c.to_ascii_lowercase() {
            'a' => Some(Direction::West),
            'd' => Some(Direction::East),
            'w' => Some(Direction::North),
            's' => Some(Direction::South),
            _ => None,
        },
        _ => None,
    }
}

fn read_key() -> Option<char> {
    let mut line = String::new();
    loop {
        print!("> ");
        stdout().lock().flush().expect("i/o error");
        if io::stdin().lock().read_line(&mut line).ok()? == 0 {
            return None;
        }
        if let Some(c) = line.trim().chars().next() {
            return Some(c);
        }
    }
}

fn main() {
    let mut grid = Grid::from_ascii(
        "
        BA IS YO .. RO IS PU
        .. .. .. .. .. .. ..
        wa wa wa wa wa .. ..
        ba .. .. .. wa .. fl
        WA IS ST .. wa .. ..
        .. .. .. .. wa .. ..
        wa wa wa wa wa .. ..
        .. .. .. .. .. .. ..
        FL IS WI .. .. .. ..",
    );
    println!("{}", grid.to_ascii());
    'foo: loop {
        match match_direction(read_key()) {
            Some(dir) => {
                let result = Turn::new(&mut grid, dir).run();
                println!("{}", grid.to_ascii());
                if result == TurnResult::Win {
                    println!("You win!");
                    break 'foo;
                }
            }
            None => {
                break 'foo;
            }
        }
    }
}
