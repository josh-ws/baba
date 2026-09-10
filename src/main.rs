use macroquad::{
    input::{KeyCode, is_key_pressed},
    window::next_frame,
};

use crate::view::Viewer;
use baba::{game::Game, pack::Levelpack, world::Direction};

mod view;

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

    let mut viewer = Viewer::new().await;
    loop {
        let mut events = Vec::new();
        if let Some(dir) = get_pressed_direction() {
            events = game.update(dir);
        }
        if is_key_pressed(KeyCode::Z) {
            game.current_level_mut().undo();
        }
        if is_key_pressed(KeyCode::Enter) {
            game.enter_link();
        }
        if is_key_pressed(KeyCode::Backspace) {
            game.return_to_root();
        }
        if is_key_pressed(KeyCode::F5) {
            match std::fs::read_to_string("assets/packs/demo.txt") {
                Ok(src) => {
                    game.reload(&src);
                }
                Err(e) => eprintln!("reload failed: {e}"),
            }
        }
        viewer.update(&game, &events);
        viewer.draw(&game);
        next_frame().await;
    }
}
