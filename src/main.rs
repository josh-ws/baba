use macroquad::{
    input::{KeyCode, is_key_pressed},
    window::next_frame,
};

use crate::view::Viewer;
use baba::{game::Game, pack::Levelpack, world::Direction};

mod view;

#[derive(Clone, Copy)]
enum Action {
    Move(Direction),
    Undo,
    EnterLevel,
    BackOutOfLevel,
    Exit,
    Refresh,
}

const KEYMAP: &[(KeyCode, Action)] = &[
    (KeyCode::W, Action::Move(Direction::North)),
    (KeyCode::A, Action::Move(Direction::West)),
    (KeyCode::S, Action::Move(Direction::South)),
    (KeyCode::D, Action::Move(Direction::East)),
    (KeyCode::Up, Action::Move(Direction::North)),
    (KeyCode::Left, Action::Move(Direction::West)),
    (KeyCode::Down, Action::Move(Direction::South)),
    (KeyCode::Right, Action::Move(Direction::East)),
    (KeyCode::Z, Action::Undo),
    (KeyCode::Enter, Action::EnterLevel),
    (KeyCode::Backspace, Action::BackOutOfLevel),
    (KeyCode::Escape, Action::Exit),
    (KeyCode::F5, Action::Refresh),
];

fn get_action() -> Option<Action> {
    for (key, action) in KEYMAP {
        if is_key_pressed(*key) {
            return Some(*action);
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
        match get_action() {
            Some(Action::Move(dir)) => {
                events = game.update(dir);
            }
            Some(Action::Undo) => {
                game.current_level_mut().undo();
            }
            Some(Action::EnterLevel) => {
                game.enter_link();
            }
            Some(Action::BackOutOfLevel) => game.return_to_root(),
            Some(Action::Refresh) => match std::fs::read_to_string("assets/packs/demo.txt") {
                Ok(src) => {
                    game.reload(&src);
                }
                Err(e) => eprintln!("reload failed: {e}"),
            },
            Some(Action::Exit) => return,
            _ => (),
        }
        viewer.update(&game, &events);
        viewer.draw(&game);
        next_frame().await;
    }
}
