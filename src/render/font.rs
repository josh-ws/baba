use macroquad::{
    color::Color,
    math::{Rect, Vec2},
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

pub struct Font {
    texture: Texture2D,
}

impl Font {
    const WIDTH: f32 = 8.;
    const HEIGHT: f32 = 12.;
    const ALPH: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    pub fn new(texture: Texture2D) -> Self {
        Self { texture }
    }

    pub fn glyph_size(&self, scale: f32) -> Vec2 {
        Vec2::new(Font::WIDTH, Font::HEIGHT) * scale
    }

    pub fn draw(&self, x: f32, y: f32, scale: f32, text: &str, color: Color) {
        let size = Vec2::new(Self::WIDTH, Self::HEIGHT) * scale;
        let mut curr_x = x;
        for c in text.chars() {
            if c.is_whitespace() {
                curr_x += 12.;
                continue;
            }
            if let Some(index) = Self::ALPH.find(c.to_ascii_uppercase()) {
                let source = Some(Rect {
                    x: index as f32 * Self::WIDTH,
                    y: 0.,
                    w: Self::WIDTH,
                    h: Self::HEIGHT,
                });
                let params = DrawTextureParams {
                    source,
                    dest_size: Some(size),
                    ..Default::default()
                };
                draw_texture_ex(&self.texture, curr_x, y, color, params);
            }
            curr_x += size.x + 3.;
        }
    }
}
