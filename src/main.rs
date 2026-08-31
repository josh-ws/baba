use std::io::{self, BufRead, Write, stdout};

use crate::{
    eval::TurnResult,
    level::Level,
    world::{Direction, Grid},
};

mod ascii;
mod eval;
mod level;
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

fn clear() {
    print!("\x1B[2J\x1B[1;1H");
}

fn main() {
    let data = include_str!("../assets/levels/demo.txt");
    let mut level = Level::read(data);
    clear();
    println!("{}", level.grid().to_ascii());
    'foo: loop {
        match match_direction(read_key()) {
            Some(dir) => match level.update(dir) {
                TurnResult::Win => {
                    println!("You win!");
                    break 'foo;
                }
                TurnResult::Continue => {
                    clear();
                    println!("{}", level.grid().to_ascii());
                }
            },
            None => break 'foo,
        }
    }
}
