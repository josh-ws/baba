use baba::{
    sprite::sprite_frame,
    unit::{Atlas, Unit},
    world::{Grid, Pos},
};
use macroquad::{
    color::WHITE,
    math::{Rect, Vec2},
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    game::Resources,
    render::layout::{Layout, TILE_SIZE},
};

pub struct DrawUnitParams<'a> {
    pub grid: &'a Grid,
    pub unit: &'a Unit,
    pub layout: &'a Layout,
    pub target: Pos,
    pub wobble: usize,
    pub offset_y: f64,
}

pub struct UnitRenderer {
    sprites: Texture2D,
    words: Texture2D,
    tiles: Texture2D,
}

impl UnitRenderer {
    pub fn new(resources: &Resources) -> Self {
        Self {
            sprites: resources.sprites.clone(),
            words: resources.words.clone(),
            tiles: resources.tiles.clone(),
        }
    }

    pub fn draw(&self, params: &DrawUnitParams) {
        let DrawUnitParams {
            grid,
            unit,
            layout,
            target,
            wobble,
            offset_y,
        } = params;
        let frame = sprite_frame(grid, unit, *target, *wobble);
        let texture = self.atlas_texture(&frame.atlas);
        let params = DrawTextureParams {
            source: Some(Rect {
                x: frame.column as f32 * TILE_SIZE,
                y: frame.row as f32 * TILE_SIZE,
                w: TILE_SIZE,
                h: TILE_SIZE,
            }),
            dest_size: Some(Vec2::new(layout.tile(), layout.tile())),
            flip_x: frame.flip_x,
            ..Default::default()
        };
        let at = layout.screen_position(*target);
        draw_texture_ex(texture, at.x, at.y - *offset_y as f32, WHITE, params);
    }

    fn atlas_texture(&self, atlas: &Atlas) -> &Texture2D {
        match atlas {
            Atlas::Sprites => &self.sprites,
            Atlas::Words => &self.words,
            Atlas::Tiled => &self.tiles,
        }
    }
}
