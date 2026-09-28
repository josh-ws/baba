use std::{collections::HashMap, f64};

use baba::{
    level::Link,
    rule::Rules,
    sprite::sprite_frame,
    unit::{Atlas, Property, Unit, lookup_unit},
    world::{Grid, Pos},
};
use macroquad::{
    color::{Color, WHITE},
    math::{Rect, Vec2},
    shapes::draw_rectangle,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    game::Resources,
    render::{
        font::Font,
        layout::{Layout, TILE_SIZE},
    },
};

const GRID_COLOR: Color = Color::new(0.0, 0.0, 0.02, 1.);

const WOBBLE_PERIOD: f64 = 0.20;
const WOBBLE_FRAMES: usize = 3;

const FLOAT_PERIOD: f64 = 3.5;
const FLOAT_HEIGHT: f64 = 0.35;

pub struct GridRenderer {
    font: Font,
    sprites: Texture2D,
    words: Texture2D,
    tiles: Texture2D,
}

impl GridRenderer {
    pub fn new(resources: &Resources) -> Self {
        Self {
            font: Font::new(resources.font.clone()),
            sprites: resources.sprites.clone(),
            words: resources.words.clone(),
            tiles: resources.tiles.clone(),
        }
    }

    pub fn draw_backdrop(&self, layout: &Layout) {
        let origin = layout.origin();
        let grid_size = layout.grid_size();
        draw_rectangle(origin.x, origin.y, grid_size.x, grid_size.y, GRID_COLOR);
    }

    pub fn draw(&self, grid: &Grid, rules: &Rules, links: &HashMap<u64, Link>, layout: &Layout, time: f64) {
        let wobble = wobble(time);
        let lift = float_lift(time) * layout.tile() as f64;
        let mut units = grid
            .units_with_pos()
            .map(|(pos, unit)| (pos, unit, rules.unit_has_prop(unit.noun(), Property::Float)))
            .collect::<Vec<(Pos, &Unit, bool)>>();
        units.sort_unstable_by_key(|(_, unit, float)| (*float, lookup_unit(unit.kind()).group));
        for (pos, unit, float) in units {
            let offset = if float { lift } else { 0. };
            self.draw_unit(grid, unit, pos, layout, wobble, offset);
            if let Some(link) = links.get(&unit.id()) {
                self.draw_link_label(link.display(), pos, layout, offset);
            }
        }
    }

    fn draw_link_label(&self, display: &str, pos: Pos, layout: &Layout, offset: f64) {
        let glyph = self.font.glyph_size(layout.scale());
        let at = layout.screen_position(pos) + ((Vec2::splat(layout.tile()) - glyph) / 2.).floor();
        self.font
            .draw(at.x, at.y - offset as f32, layout.scale(), display, WHITE);
    }

    fn draw_unit(&self, grid: &Grid, unit: &Unit, pos: Pos, layout: &Layout, wobble: usize, offset_y: f64) {
        let frame = sprite_frame(grid, unit, pos, wobble);
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
        let at = layout.screen_position(pos);
        draw_texture_ex(texture, at.x, at.y - offset_y as f32, WHITE, params);
    }

    fn atlas_texture(&self, atlas: &Atlas) -> &Texture2D {
        match atlas {
            Atlas::Sprites => &self.sprites,
            Atlas::Words => &self.words,
            Atlas::Tiled => &self.tiles,
        }
    }
}

fn wobble(time: f64) -> usize {
    (time / WOBBLE_PERIOD) as usize % WOBBLE_FRAMES
}

fn float_lift(time: f64) -> f64 {
    let phase = (time / FLOAT_PERIOD * f64::consts::TAU).sin();
    FLOAT_HEIGHT * (1. + phase) / 2.
}
