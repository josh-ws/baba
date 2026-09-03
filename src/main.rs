use macroquad::{
    input::{KeyCode, is_key_pressed},
    window::next_frame,
};

use crate::{
    eval::{TurnResult, TurnStatus},
    pack::Levelpack,
    view::Viewer,
    world::Direction,
};

mod ascii;
mod eval;
mod level;
mod lex;
mod pack;
mod rule;
mod unit;
mod view;
mod world;

const KEYMAP: &[(KeyCode, Direction)] = &[
    (KeyCode::W, Direction::North),
    (KeyCode::A, Direction::West),
    (KeyCode::S, Direction::South),
    (KeyCode::D, Direction::East),
];

fn get_pressed_direction() -> Option<Direction> {
    for (key, direction) in KEYMAP {
        if is_key_pressed(*key) {
            return Some(*direction);
        }
    }
    None
}

#[macroquad::main("baba")]
async fn main() {
    let mut pack = Levelpack::parse(include_str!("../assets/packs/demo.txt"));
    let mut current_level = "map".to_string();

    let viewer = Viewer::new().await;
    let mut turn_result = TurnResult::new(TurnStatus::Continue, vec![]);
    loop {
        if let Some(dir) = get_pressed_direction() {
            turn_result = pack.get_level_mut(&current_level).update(dir);
        }
        if is_key_pressed(KeyCode::Enter) {
            if let Some(key) = pack.get_level(&current_level).caption(&turn_result.selected) {
                current_level = key.to_string();
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            current_level = "map".to_string();
        }
        viewer.draw(pack.get_level(&current_level), &turn_result.selected);
        next_frame().await;
        if turn_result.status == TurnStatus::Win {
            println!("You win!");
            break;
        }
    }
}
