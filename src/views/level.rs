use std::f64;

use macroquad::{
    color::{Color, WHITE},
    input::{KeyCode, is_key_pressed},
    math::{Rect, Vec2},
    rand,
    shapes::draw_rectangle,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
    time::{draw_fps, get_time},
    window::{clear_background, screen_height, screen_width},
};

use baba::{
    eval::{Cause, Event},
    level::Level,
    lex::lex,
    parse::parse,
    rule::Rules,
    scenes::level::{LevelAction, LevelScene},
    unit::{Atlas, Facing, Noun, Property, Unit, lookup_unit},
    world::{
        Direction::{self},
        Grid, Pos,
    },
};

use crate::game::Resources;

const TILE_SIZE: f32 = 24.0;
const BACKGROUND_COLOR: Color = Color::new(0.1, 0.1, 0.2, 1.);
const GRID_COLOR: Color = Color::new(0.0, 0.0, 0.02, 1.);

const WOBBLE_PERIOD: f64 = 0.20;
const WOBBLE_FRAMES: usize = 3;

const FLOAT_PERIOD: f64 = 3.5;
const FLOAT_HEIGHT: f64 = 0.35;

const PARTICLE_SPAWN_PERIOD: f64 = 0.3;

const KEYMAP: &[(KeyCode, LevelAction)] = &[
    (KeyCode::W, LevelAction::Move(Direction::North)),
    (KeyCode::A, LevelAction::Move(Direction::West)),
    (KeyCode::S, LevelAction::Move(Direction::South)),
    (KeyCode::D, LevelAction::Move(Direction::East)),
    (KeyCode::Up, LevelAction::Move(Direction::North)),
    (KeyCode::Left, LevelAction::Move(Direction::West)),
    (KeyCode::Down, LevelAction::Move(Direction::South)),
    (KeyCode::Right, LevelAction::Move(Direction::East)),
    (KeyCode::Z, LevelAction::Undo),
    (KeyCode::Enter, LevelAction::EnterLevel),
    (KeyCode::Backspace, LevelAction::BackOutOfLevel),
    (KeyCode::Space, LevelAction::Idle),
];

struct Font {
    texture: Texture2D,
}

impl Font {
    const WIDTH: f32 = 8.;
    const HEIGHT: f32 = 12.;
    const ALPH: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    fn new(resources: &Resources) -> Self {
        Self {
            texture: resources.font.clone(),
        }
    }

    fn draw(&self, x: f32, y: f32, scale: f32, text: &str, color: Color) {
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

#[derive(Clone, Copy)]
enum ParticleKind {
    Sparkle,
    Splash,
    Explode,
    Steam,
    Melt,
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
        }
    }

    fn frame(&self, now: f64) -> u64 {
        ((now - self.birth) / self.period) as u64
    }

    fn alive(&self, now: f64) -> bool {
        self.frame(now) < self.frames
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

    fn screen_point(&self, point: Vec2) -> Vec2 {
        self.origin + point * self.tile
    }

    fn screen_position(&self, pos: Pos) -> Vec2 {
        self.screen_point(Vec2::new(pos.x as f32, pos.y as f32))
    }
}

pub struct LevelView {
    key: String,
    resources: Resources,
    particles: Vec<Particle>,
    next_spawn: f64,
    font: Font,
}

impl LevelView {
    pub fn new(resources: &Resources) -> Self {
        LevelView {
            key: String::default(),
            resources: resources.clone(),
            particles: Vec::new(),
            next_spawn: 0.,
            font: Font::new(resources),
        }
    }

    pub fn input(&self) -> Option<LevelAction> {
        for (key, action) in KEYMAP {
            if is_key_pressed(*key) {
                return Some(*action);
            }
        }
        None
    }

    pub fn update(&mut self, scene: &LevelScene, events: &[Event]) {
        let switched = scene.current_key() != self.key;
        if switched {
            self.key = scene.current_key().to_string();
            self.particles.clear();
        }

        let time = get_time();
        if time >= self.next_spawn {
            let grid = scene.current_level().grid();
            let rules = parse(&lex(grid));
            self.spawn_ambient(grid, &rules, time);
            self.next_spawn = time + PARTICLE_SPAWN_PERIOD;
        }
        if !switched {
            for event in events {
                match event {
                    Event::Destroyed { pos, cause } => {
                        let kind = match cause {
                            Cause::Defeat => ParticleKind::Explode,
                            Cause::Sink => ParticleKind::Splash,
                            Cause::Melt => ParticleKind::Melt,
                        };
                        self.burst(kind, *pos, time);
                    }
                }
            }
        }
        self.particles.retain(|p| p.alive(time));
    }

    pub fn draw(&self, scene: &LevelScene) {
        draw_fps();
        let time = get_time();
        let grid = scene.current_level().grid();
        let rules = parse(&lex(grid));
        let layout = Layout::new(grid, Vec2::new(screen_width(), screen_height()));
        self.draw_background(&layout);
        self.draw_caption(scene);
        self.draw_units(&rules, scene.current_level(), &layout, time);
        self.draw_particles(&layout, time);
    }

    fn draw_background(&self, layout: &Layout) {
        let Layout { origin, grid_size, .. } = layout;
        clear_background(BACKGROUND_COLOR);
        draw_rectangle(origin.x, origin.y, grid_size.x, grid_size.y, GRID_COLOR);
    }

    fn draw_caption(&self, scene: &LevelScene) {
        if let Some(caption) = scene.caption() {
            let scale = 2.;
            self.font.draw(0., 0., scale, caption, WHITE);
        }
    }

    fn draw_units(&self, rules: &Rules, level: &Level, layout: &Layout, time: f64) {
        let wobble = wobble(time);
        let lift = float_lift(time) * layout.tile as f64;
        let mut units = level
            .grid()
            .units_with_pos()
            .map(|(pos, unit)| (pos, unit, rules.unit_has_prop(unit.noun(), Property::Float)))
            .collect::<Vec<(Pos, &Unit, bool)>>();
        units.sort_unstable_by_key(|(_, unit, float)| (*float, lookup_unit(unit.kind()).group));
        for (pos, unit, float) in units {
            let offset = if float { lift } else { 0. };
            self.draw_unit(level.grid(), unit, pos, layout, wobble, offset);
            if let Some(link) = level.links().get(&unit.id()) {
                self.draw_link_label(link.display(), pos, layout, offset);
            }
        }
    }

    fn draw_link_label(&self, display: &str, pos: Pos, layout: &Layout, offset: f64) {
        let scale = layout.tile / TILE_SIZE;
        let glyph = Vec2::new(Font::WIDTH, Font::HEIGHT) * scale;
        let at = layout.screen_position(pos) + ((Vec2::splat(layout.tile) - glyph) / 2.).floor();
        self.font
            .draw(at.x, at.y - offset as f32, layout.tile / TILE_SIZE, display, WHITE);
    }

    fn draw_unit(&self, grid: &Grid, unit: &Unit, pos: Pos, layout: &Layout, wobble: usize, offset_y: f64) {
        let data = lookup_unit(unit.kind());
        let texture = self.atlas_texture(&data.sprite.atlas);
        let (column, flip_x) = sprite_column(grid, unit, pos, wobble);
        let row = sprite_row(unit, wobble);
        let params = DrawTextureParams {
            source: Some(Rect {
                x: column as f32 * TILE_SIZE,
                y: row as f32 * TILE_SIZE,
                w: TILE_SIZE,
                h: TILE_SIZE,
            }),
            dest_size: Some(Vec2::new(layout.tile, layout.tile)),
            flip_x,
            ..Default::default()
        };
        let at = layout.screen_position(pos);
        draw_texture_ex(texture, at.x, at.y - offset_y as f32, WHITE, params);
    }

    fn atlas_texture(&self, atlas: &Atlas) -> &Texture2D {
        match atlas {
            Atlas::Sprites => &self.resources.sprites,
            Atlas::Words => &self.resources.words,
            Atlas::Tiled => &self.resources.tiles,
        }
    }

    fn spawn_ambient(&mut self, grid: &Grid, rules: &Rules, time: f64) {
        self.spawn_ambient_from_prop(grid, rules, Property::Win, ParticleKind::Sparkle, time);
        self.spawn_ambient_from_prop(grid, rules, Property::Hot, ParticleKind::Steam, time);
    }

    fn spawn_ambient_from_prop(&mut self, grid: &Grid, rules: &Rules, prop: Property, kind: ParticleKind, time: f64) {
        let win_cells = grid
            .cells_with_pos()
            .filter(|(_, cell)| cell.units().iter().any(|unit| rules.unit_has_prop(unit.noun(), prop)));
        for (pos, _) in win_cells {
            self.burst(kind, pos, time);
        }
    }

    fn burst(&mut self, kind: ParticleKind, pos: Pos, time: f64) {
        let (count, offset, chance) = match kind {
            ParticleKind::Sparkle => (1, 0.5, 1.0),
            ParticleKind::Splash => (10, 0.5, 1.0),
            ParticleKind::Explode => (20, 0.6, 1.0),
            ParticleKind::Steam => (1, 0.5, 0.05),
            ParticleKind::Melt => (1, 0., 1.0),
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

    fn draw_particles(&self, layout: &Layout, time: f64) {
        let texture = &self.resources.particles;
        let scale = layout.tile / TILE_SIZE;
        for particle in &self.particles {
            let frame = particle.frame(time).min(particle.frames - 1) as f32;
            let dest_size = particle.tile_size * scale;
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

fn wobble(time: f64) -> usize {
    (time / WOBBLE_PERIOD) as usize % WOBBLE_FRAMES
}

fn float_lift(time: f64) -> f64 {
    let phase = (time / FLOAT_PERIOD * f64::consts::TAU).sin();
    FLOAT_HEIGHT * (1. + phase) / 2.
}

fn direction_index(direction: Direction) -> usize {
    match direction {
        Direction::East | Direction::West => 0,
        Direction::South => 1,
        Direction::North => 2,
    }
}

// TODO(jw) PERF: Should this be a cached property on the unit?
fn tiled_index(grid: &Grid, pos: Pos, noun: Noun) -> usize {
    const DIRS: [Direction; 4] = [Direction::East, Direction::North, Direction::West, Direction::South];
    let mut index = 0;
    for (i, dir) in DIRS.iter().enumerate() {
        let shift = pos.shift(*dir);
        let joins = || {
            grid.at(shift)
                .units()
                .iter()
                .any(|u| u.noun() == noun || u.noun() == Noun::Level)
        };
        if !grid.in_bounds(shift) || joins() {
            index |= 1 << i;
        }
    }
    index
}

/// Returns column from the spritesheet for this sprite, and whether or not it should be drawn flipped
fn sprite_column(grid: &Grid, unit: &Unit, pos: Pos, wobble: usize) -> (usize, bool) {
    let data = lookup_unit(unit.kind());
    match data.sprite.atlas {
        Atlas::Words => (wobble, false),
        Atlas::Tiled => (tiled_index(grid, pos, unit.noun()), false),
        Atlas::Sprites => {
            let (dir, flip) = sprite_facing(unit.direction(), data.sprite.facing);
            (wobble * 3 + dir, flip)
        }
    }
}

fn sprite_row(unit: &Unit, wobble: usize) -> usize {
    let data = lookup_unit(unit.kind());
    match data.sprite.atlas {
        Atlas::Sprites | Atlas::Words => data.sprite.row,
        Atlas::Tiled => data.sprite.row + wobble,
    }
}

fn sprite_facing(direction: Direction, facing: Facing) -> (usize, bool) {
    match facing {
        Facing::Fixed => (0, false),
        Facing::Directional => (direction_index(direction), direction == Direction::West),
    }
}
