use std::f64;

use macroquad::{
    color::{Color, WHITE},
    input::{KeyCode, is_key_pressed},
    math::Vec2,
    time::{draw_fps, get_time},
    window::{clear_background, screen_height, screen_width},
};

use baba::{
    eval::{Cause, Event},
    lex::lex,
    parse::parse,
    rule::Rules,
    scenes::level::{LevelAction, LevelScene},
    unit::Property,
    world::{
        Direction::{self},
        Grid,
    },
};

use crate::{
    game::Resources,
    render::{
        font::Font,
        grid::GridRenderer,
        layout::Layout,
        particles::{ParticleKind, Particles},
    },
};

const BACKGROUND_COLOR: Color = Color::new(0.1, 0.1, 0.2, 1.);
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

pub struct LevelView {
    key: String,
    grid_renderer: GridRenderer,
    particle_renderer: Particles,
    next_spawn: f64,
    font: Font,
}

impl LevelView {
    pub fn new(resources: &Resources) -> Self {
        let font = resources.font.clone();
        LevelView {
            key: String::default(),
            grid_renderer: GridRenderer::new(resources),
            particle_renderer: Particles::new(resources),
            next_spawn: 0.,
            font: Font::new(font),
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
            self.particle_renderer.clear();
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
                    Event::Created { pos: _ } => (),
                    Event::Destroyed { pos, cause } => {
                        let kind = match cause {
                            Cause::Defeat => ParticleKind::Explode,
                            Cause::Sink => ParticleKind::Splash,
                            Cause::Melt => ParticleKind::Melt,
                            Cause::Open => ParticleKind::Open,
                        };
                        self.particle_renderer.burst(kind, *pos, time);
                    }
                }
            }
        }
        self.particle_renderer.retain(time);
    }

    pub fn draw(&self, scene: &LevelScene) {
        let time = get_time();
        let grid = scene.current_level().grid();
        let links = scene.current_level().links();
        let rules = parse(&lex(grid));
        let layout = Layout::new(grid, Vec2::new(screen_width(), screen_height()));

        clear_background(BACKGROUND_COLOR);
        draw_fps();
        self.grid_renderer.draw_backdrop(&layout);
        self.grid_renderer.draw(grid, &rules, links, &layout, time);
        self.draw_caption(scene);
        self.particle_renderer.draw(&layout, time);
    }

    fn draw_caption(&self, scene: &LevelScene) {
        if let Some(caption) = scene.caption() {
            let scale = 2.;
            self.font.draw(0., 0., scale, caption, WHITE);
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
            self.particle_renderer.burst(kind, pos, time);
        }
    }
}
