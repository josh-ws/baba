use macroquad::{
    input::{KeyCode, is_key_pressed},
    window::next_frame,
};

use crate::view::Viewer;
use baba::{game::Game, world::Direction};

mod view;

const PACK_SRC: &str = "assets/packs/demo.txt"; // TODO(jw) move hardcoded path

#[derive(Clone, Copy)]
enum Action {
    Move(Direction),
    Undo,
    EnterLevel,
    BackOutOfLevel,
    Exit,
    Refresh,
    Idle,
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
    (KeyCode::Space, Action::Idle),
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
async fn main() -> Result<(), String> {
    let mut game = Game::from_file(PACK_SRC)?;

    let mut viewer = Viewer::new().await?;
    loop {
        let mut events = Vec::new();
        match get_action() {
            Some(Action::Move(dir)) => {
                events = game.update(Some(dir));
            }
            Some(Action::Undo) => {
                game.undo();
            }
            Some(Action::EnterLevel) => {
                game.enter_link();
            }
            Some(Action::BackOutOfLevel) => game.return_to_parent(),
            Some(Action::Refresh) => {
                if let Err(e) = game.reload(PACK_SRC) {
                    eprintln!("could not reload pack: {e}")
                }
            }
            Some(Action::Exit) => return Ok(()),
            Some(Action::Idle) => {
                events = game.update(None);
            }
            None => (),
        }
        viewer.update(&game, &events);
        viewer.draw(&game);
        next_frame().await;
    }
}
