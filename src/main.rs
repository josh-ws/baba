use macroquad::{
    input::{KeyCode, is_key_pressed},
    window::next_frame,
};

use crate::{game::Game, pack::Levelpack, view::Viewer, world::Direction};

mod ascii;
mod eval;
mod game;
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
    let pack = Levelpack::parse(include_str!("../assets/packs/demo.txt"));
    let mut game = Game::new(pack);

    let viewer = Viewer::new().await;
    loop {
        if let Some(dir) = get_pressed_direction() {
            game.update(dir);
        }
        if is_key_pressed(KeyCode::Enter) {
            game.enter_link();
        }
        if is_key_pressed(KeyCode::Backspace) {
            game.return_to_root();
        }
        viewer.draw(&game);
        next_frame().await;
    }
}
