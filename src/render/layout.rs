use baba::world::{Grid, Pos};
use macroquad::math::Vec2;

pub const TILE_SIZE: f32 = 24.0;

pub struct Layout {
    origin: Vec2,    // where the level is drawn
    grid_size: Vec2, // total size, in pixels, of the level grid
    tile: f32,       // size in pixels of each destination tile
}

impl Layout {
    pub fn new(grid: &Grid, screen_size: Vec2) -> Self {
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

    pub fn screen_point(&self, point: Vec2) -> Vec2 {
        self.origin + point * self.tile
    }

    pub fn screen_position(&self, pos: Pos) -> Vec2 {
        self.screen_point(Vec2::new(pos.x as f32, pos.y as f32))
    }

    pub fn scale(&self) -> f32 {
        self.tile / TILE_SIZE
    }

    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn grid_size(&self) -> Vec2 {
        self.grid_size
    }

    pub fn tile(&self) -> f32 {
        self.tile
    }
}
