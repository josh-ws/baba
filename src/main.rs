use baba::scenes::{Scene, level::LevelScene};
use macroquad::{
    input::{KeyCode, is_key_pressed},
    window::{Conf, next_frame},
};

use crate::game::{Game, Resources};

mod game;
mod views;

const PACK_SRC: &str = "assets/packs/demo.txt"; // TODO(jw) move hardcoded path

#[derive(Clone, Copy)]
enum Action {
    Exit,
    Refresh,
}

const KEYMAP: &[(KeyCode, Action)] = &[(KeyCode::F5, Action::Refresh), (KeyCode::Escape, Action::Exit)];

fn get_action() -> Option<Action> {
    for (key, action) in KEYMAP {
        if is_key_pressed(*key) {
            return Some(*action);
        }
    }
    None
}

fn window_conf() -> Conf {
    Conf {
        window_title: "baba".into(),
        window_width: 1200,
        window_height: 1200,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), String> {
    let assets = Resources::load().await?;
    let scene = LevelScene::from_file(PACK_SRC)?;
    let mut game = Game::new(assets, Scene::Level(scene));
    loop {
        match get_action() {
            Some(Action::Refresh) => game.reload(PACK_SRC),
            Some(Action::Exit) => return Ok(()),
            None => (),
        }
        game.update();
        game.draw();
        next_frame().await;
    }
}
