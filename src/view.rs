use macroquad::{
    color::{BLACK, LIGHTGRAY, WHITE},
    input::KeyCode,
    input::is_key_pressed,
    math::{Rect, Vec2},
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex, load_texture},
    window::{clear_background, next_frame},
};

use crate::{
    eval::TurnResult,
    level::Level,
    unit::{Atlas, Facing, Unit, lookup_unit},
    world::{
        Direction::{self},
        Pos,
    },
};

const TILE_SIZE: f32 = 24.0;
const SCALE: f32 = 2.0;
const DEST_TILE_SIZE: f32 = TILE_SIZE * SCALE;

const KEYMAP: &[(KeyCode, Direction)] = &[
    (KeyCode::W, Direction::North),
    (KeyCode::A, Direction::West),
    (KeyCode::S, Direction::South),
    (KeyCode::D, Direction::East),
];

fn direction_index(direction: Direction) -> usize {
    match direction {
        Direction::East | Direction::West => 0,
        Direction::South => 1,
        Direction::North => 2,
    }
}

pub struct Viewer {
    level: Level,
    status: TurnResult,
    sprites: Texture2D,
    words: Texture2D,
}

impl Viewer {
    pub async fn new(level: Level) -> Self {
        Viewer {
            level,
            status: TurnResult::Continue,
            sprites: load_texture("assets/sprites.png").await.unwrap(),
            words: load_texture("assets/words.png").await.unwrap(),
        }
    }

    fn get_pressed_direction() -> Option<Direction> {
        for (key, direction) in KEYMAP {
            if is_key_pressed(*key) {
                return Some(*direction);
            }
        }
        None
    }

    fn draw_unit(&self, unit: &Unit, pos: Pos) {
        let data = lookup_unit(unit.kind());
        let (direction_index, flip_x) = match data.sprite.facing {
            Facing::Fixed => (0, false),
            Facing::Directional => (
                direction_index(unit.direction()),
                unit.direction() == Direction::West,
            ),
        };
        let t = match data.sprite.atlas {
            Atlas::Sprites => &self.sprites,
            Atlas::Words => &self.words,
        };
        let source = Rect {
            x: direction_index as f32 * TILE_SIZE,
            y: data.sprite.row as f32 * TILE_SIZE,
            w: TILE_SIZE,
            h: TILE_SIZE,
        };
        let dest = Vec2::new(DEST_TILE_SIZE, DEST_TILE_SIZE);
        let params = DrawTextureParams {
            source: Some(source),
            dest_size: Some(dest),
            flip_x,
            ..Default::default()
        };
        draw_texture_ex(
            t,
            pos.x as f32 * DEST_TILE_SIZE,
            pos.y as f32 * DEST_TILE_SIZE,
            WHITE,
            params,
        );
    }

    pub fn status(&self) -> &TurnResult {
        &self.status
    }

    pub fn update(&mut self) {
        if self.status == TurnResult::Win {
            return;
        }
        self.status = match Viewer::get_pressed_direction() {
            Some(direction) => self.level.update(direction),
            _ => TurnResult::Continue,
        };
    }

    pub fn draw(&self) {
        clear_background(BLACK);
        draw_text(self.level.name(), 0.0, 20.0, 30.0, LIGHTGRAY);

        for (pos, unit) in self.level.grid().units() {
            self.draw_unit(unit, pos);
        }
    }
}

pub async fn run_game(level_src: &str) {
    let level = Level::read(level_src);
    let mut viewer = Viewer::new(level).await;
    loop {
        viewer.update();
        viewer.draw();
        if *viewer.status() == TurnResult::Win {
            println!("You win!");
            break;
        }
        next_frame().await;
    }
}
