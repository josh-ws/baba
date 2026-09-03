use macroquad::{
    color::{Color, LIGHTGRAY, WHITE},
    input::{KeyCode, is_key_pressed},
    math::{Rect, Vec2},
    miniquad::window::set_window_size,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{DrawTextureParams, FilterMode, Texture2D, draw_texture_ex, load_texture},
    time::get_time,
    window::{clear_background, next_frame, screen_height, screen_width},
};

use crate::{
    eval::TurnResult,
    level::Level,
    unit::{Atlas, Facing, Unit, lookup_unit},
    world::{
        Direction::{self},
        Grid, Pos,
    },
};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 820;
const TILE_SIZE: f32 = 24.0;
const BACKGROUND_COLOR: Color = Color::new(0.1, 0.2, 0.2, 1.);
const GRID_COLOR: Color = Color::new(0.1, 0.25, 0.25, 1.);
const WOBBLE_PERIOD: f64 = 0.20;
const WOBBLE_FRAMES: usize = 3;

const KEYMAP: &[(KeyCode, Direction)] = &[
    (KeyCode::W, Direction::North),
    (KeyCode::A, Direction::West),
    (KeyCode::S, Direction::South),
    (KeyCode::D, Direction::East),
];

fn wobble(time: f64) -> usize {
    (time / WOBBLE_PERIOD) as usize % WOBBLE_FRAMES
}

fn direction_index(direction: Direction) -> usize {
    match direction {
        Direction::East | Direction::West => 0,
        Direction::South => 1,
        Direction::North => 2,
    }
}

struct Layout {
    origin: Vec2,    // where the level is drawn
    grid_size: Vec2, // total size, in pixels, of the level grid
    tile: f32,       // size in pixels of each destination tile
}

impl Layout {
    fn new(grid: &Grid, screen_size: Vec2) -> Self {
        debug_assert!(grid.width() > 0);
        let fit = (screen_size / Vec2::new(grid.width() as f32, grid.height() as f32)).min_element();
        let tile = (fit / TILE_SIZE).floor().max(1.) * TILE_SIZE;
        let grid_size = Vec2::new(grid.width() as f32, grid.height() as f32) * tile;
        Self {
            origin: ((screen_size - grid_size) / 2.).floor(),
            grid_size,
            tile,
        }
    }

    fn screen_position(&self, pos: Pos) -> Vec2 {
        self.origin + Vec2::new(pos.x as f32, pos.y as f32) * self.tile
    }
}

pub struct Viewer {
    sprites: Texture2D,
    words: Texture2D,
}

impl Viewer {
    pub async fn new() -> Self {
        set_window_size(WINDOW_WIDTH, WINDOW_HEIGHT);
        let sprites = load_texture("assets/sprites.png").await.unwrap();
        let words = load_texture("assets/words.png").await.unwrap();
        sprites.set_filter(FilterMode::Nearest);
        words.set_filter(FilterMode::Nearest);
        Viewer { sprites, words }
    }

    pub fn draw(&self, level: &Level) {
        let layout = Layout::new(level.grid(), Vec2::new(screen_width(), screen_height()));
        let wobble = wobble(get_time());
        let Layout { grid_size, origin, .. } = layout;
        clear_background(BACKGROUND_COLOR);
        draw_rectangle(origin.x, origin.y, grid_size.x, grid_size.y, GRID_COLOR);
        draw_text(level.name(), 0.0, 20.0, 30.0, LIGHTGRAY);
        for (pos, unit) in level.grid().units() {
            self.draw_unit(unit, pos, &layout, wobble);
        }
    }

    fn draw_unit(&self, unit: &Unit, pos: Pos, layout: &Layout, wobble: usize) {
        let data = lookup_unit(unit.kind());
        let texture = self.atlas_texture(&data.sprite.atlas);
        let (column, flip_x) = sprite_column(data.sprite.atlas, data.sprite.facing, unit.direction(), wobble);
        let params = DrawTextureParams {
            source: Some(Rect {
                x: column as f32 * TILE_SIZE,
                y: data.sprite.row as f32 * TILE_SIZE,
                w: TILE_SIZE,
                h: TILE_SIZE,
            }),
            dest_size: Some(Vec2::new(layout.tile, layout.tile)),
            flip_x,
            ..Default::default()
        };
        let at = layout.screen_position(pos);
        draw_texture_ex(texture, at.x, at.y, WHITE, params);
    }

    fn atlas_texture(&self, atlas: &Atlas) -> &Texture2D {
        match atlas {
            Atlas::Sprites => &self.sprites,
            Atlas::Words => &self.words,
        }
    }
}

fn sprite_facing(direction: Direction, facing: Facing) -> (usize, bool) {
    match facing {
        Facing::Fixed => (0, false),
        Facing::Directional => (direction_index(direction), direction == Direction::West),
    }
}

fn sprite_column(atlas: Atlas, facing: Facing, direction: Direction, wobble: usize) -> (usize, bool) {
    let (dir, flip) = sprite_facing(direction, facing);
    let stride = match atlas {
        Atlas::Sprites => 3,
        Atlas::Words => 1,
    };
    (wobble * stride + dir, flip)
}

fn get_pressed_direction() -> Option<Direction> {
    for (key, direction) in KEYMAP {
        if is_key_pressed(*key) {
            return Some(*direction);
        }
    }
    None
}

pub async fn run_game(level: &mut Level) {
    let viewer = Viewer::new().await;
    loop {
        let result = match get_pressed_direction() {
            Some(dir) => level.update(dir),
            None => TurnResult::Continue,
        };
        viewer.draw(level);
        next_frame().await;
        if result == TurnResult::Win {
            println!("You win!");
            break;
        }
    }
}
