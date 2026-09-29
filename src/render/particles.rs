use baba::world::Pos;
use macroquad::{
    color::WHITE,
    math::{Rect, Vec2},
    rand,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{game::Resources, render::layout::Layout};

#[derive(Clone, Copy)]
pub enum ParticleKind {
    Sparkle,
    Splash,
    Explode,
    Steam,
    Melt,
    Open,
}

struct Particle {
    tile_size: Vec2,
    tile_offset: Vec2,
    pos: Vec2,
    birth: f64,
    frames: u64,
    period: f64,
}

impl Particle {
    fn new(kind: ParticleKind, origin: Vec2, now: f64) -> Self {
        match kind {
            ParticleKind::Sparkle => Self {
                tile_size: Vec2::new(8., 8.),
                tile_offset: Vec2::new(0., 0.),
                pos: origin,
                birth: now,
                frames: 5,
                period: 0.15,
            },
            ParticleKind::Splash => Self {
                tile_size: Vec2::new(8., 8.),
                tile_offset: Vec2::new(0., 16.),
                pos: origin,
                birth: now,
                frames: 2,
                period: 0.1,
            },
            ParticleKind::Explode => Self {
                tile_size: Vec2::new(8., 8.),
                tile_offset: Vec2::new(0., 24.),
                pos: origin,
                birth: now,
                frames: 4,
                period: 0.05,
            },
            ParticleKind::Steam => Self {
                tile_size: Vec2::new(16., 16.),
                tile_offset: Vec2::new(0., 32.),
                pos: origin,
                birth: now,
                frames: 6,
                period: 0.15,
            },
            ParticleKind::Melt => Self {
                tile_size: Vec2::new(16., 16.),
                tile_offset: Vec2::new(0., 48.),
                pos: origin,
                birth: now,
                frames: 6,
                period: 0.04,
            },
            ParticleKind::Open => Self {
                tile_size: Vec2::new(8., 8.),
                tile_offset: Vec2::new(0., 64.),
                pos: origin,
                birth: now,
                frames: 7,
                period: 0.05,
            },
        }
    }

    fn frame(&self, now: f64) -> u64 {
        ((now - self.birth) / self.period) as u64
    }

    fn alive(&self, now: f64) -> bool {
        self.frame(now) < self.frames
    }
}

pub struct Particles {
    texture: Texture2D,
    particles: Vec<Particle>,
}

impl Particles {
    pub fn new(resources: &Resources) -> Self {
        Self {
            texture: resources.particles.clone(),
            particles: Vec::new(),
        }
    }

    pub fn burst(&mut self, kind: ParticleKind, pos: Pos, time: f64) {
        let (count, offset, chance) = match kind {
            ParticleKind::Sparkle => (1, 0.5, 1.0),
            ParticleKind::Splash => (10, 0.5, 1.0),
            ParticleKind::Explode => (20, 0.6, 1.0),
            ParticleKind::Steam => (1, 0.5, 0.05),
            ParticleKind::Melt => (1, 0., 1.0),
            ParticleKind::Open => (5, 0.5, 1.0),
        };
        let centre = Vec2::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5);
        for _ in 0..count {
            let jitter = Vec2::new(rand::gen_range(-offset, offset), rand::gen_range(-offset, offset));
            let roll = rand::gen_range(0., 1.);
            if roll < chance {
                self.particles.push(Particle::new(kind, centre + jitter, time));
            }
        }
    }

    pub fn retain(&mut self, time: f64) {
        self.particles.retain(|p| p.alive(time));
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn draw(&self, layout: &Layout, time: f64) {
        let texture = &self.texture;
        for particle in &self.particles {
            let frame = particle.frame(time).min(particle.frames - 1) as f32;
            let dest_size = particle.tile_size * layout.scale();
            let params = DrawTextureParams {
                source: Some(Rect {
                    x: particle.tile_offset.x + frame * particle.tile_size.x,
                    y: particle.tile_offset.y,
                    w: particle.tile_size.x,
                    h: particle.tile_size.y,
                }),
                dest_size: Some(dest_size),
                ..Default::default()
            };
            let at = layout.screen_point(particle.pos) - dest_size / 2.;
            draw_texture_ex(texture, at.x, at.y, WHITE, params);
        }
    }
}
