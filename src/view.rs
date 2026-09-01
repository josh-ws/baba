use macroquad::{
    color::{Color, LIGHTGRAY, WHITE},
    input::{KeyCode, is_key_pressed},
    math::{Rect, Vec2},
    miniquad::window::set_window_size,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex, load_texture},
    window::{clear_background, next_frame, screen_height, screen_width},
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
        set_window_size(800, 800);
        Viewer {
            level,
            status: TurnResult::Continue,
            sprites: load_texture("assets/sprites.png").await.unwrap(),
            words: load_texture("assets/words.png").await.unwrap(),
        }
    }

    pub fn status(&self) -> &TurnResult {
        &self.status
    }

    pub fn update(&mut self) {
        self.status = match Viewer::get_pressed_direction() {
            Some(direction) => self.level.update(direction),
            _ => TurnResult::Continue,
        };
    }

    pub fn draw(&self) {
        const BACKGROUND_COLOR: Color = Color::new(0.1, 0.2, 0.2, 1.);
        const GRID_BACKGROUND_COLOR: Color = Color::new(0.1, 0.25, 0.25, 1.);

        clear_background(BACKGROUND_COLOR);
        let origin = self.origin();
        let size = self.board_size();
        draw_rectangle(origin.x, origin.y, size.x, size.y, GRID_BACKGROUND_COLOR);
        draw_text(self.level.name(), 0.0, 20.0, 30.0, LIGHTGRAY);
        for (pos, unit) in self.level.grid().units() {
            self.draw_unit(unit, pos);
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

    fn scale(&self) -> f32 {
        let grid = self.level.grid();
        let width = screen_width() / grid.width() as f32;
        let height = screen_height() / grid.height() as f32;
        let min_size = if width < height { width } else { height };
        min_size / TILE_SIZE
    }

    fn dest_tile_size(&self) -> f32 {
        self.scale() * TILE_SIZE
    }

    fn board_size(&self) -> Vec2 {
        let grid = self.level.grid();
        Vec2::new(grid.width() as f32, grid.height() as f32) * self.dest_tile_size()
    }

    fn origin(&self) -> Vec2 {
        let board_size = self.board_size();
        let screen_size = Vec2::new(screen_width(), screen_height());
        (screen_size - board_size) / 2.
    }

    fn screen_position(&self, pos: Pos) -> Vec2 {
        self.origin() + Vec2::new(pos.x as f32, pos.y as f32) * self.dest_tile_size()
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
        let dest_tile_size = self.dest_tile_size();
        let dest = Vec2::new(dest_tile_size, dest_tile_size);
        let params = DrawTextureParams {
            source: Some(source),
            dest_size: Some(dest),
            flip_x,
            ..Default::default()
        };
        let at = self.screen_position(pos);
        draw_texture_ex(t, at.x, at.y, WHITE, params);
    }
}

pub async fn run_game(level_src: &str) {
    let level = Level::read(level_src);
    let mut viewer = Viewer::new(level).await;
    while *viewer.status() != TurnResult::Win {
        viewer.update();
        viewer.draw();
        next_frame().await;
    }
    println!("You win!");
}
