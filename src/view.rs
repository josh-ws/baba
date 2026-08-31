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
    unit::{Noun, Operator, Property, Text, UnitKind},
    world::{Direction, Pos},
};

const TILE_SIZE: usize = 24;

const NOUNS: &[Noun] = &[Noun::Baba, Noun::Flag, Noun::Rock, Noun::Wall];
const TEXT: &[Text] = &[
    Text::Noun(Noun::Baba),
    Text::Noun(Noun::Flag),
    Text::Noun(Noun::Rock),
    Text::Property(Property::You),
    Text::Property(Property::Stop),
    Text::Property(Property::Push),
    Text::Property(Property::Win),
    Text::Operator(Operator::Is),
    Text::Noun(Noun::Wall),
];

const KEYMAP: &[(KeyCode, Direction)] = &[
    (KeyCode::W, Direction::North),
    (KeyCode::A, Direction::West),
    (KeyCode::S, Direction::South),
    (KeyCode::D, Direction::East),
];

fn noun_index(noun: Noun) -> Option<usize> {
    NOUNS.iter().position(|n| *n == noun)
}

fn text_index(text: Text) -> Option<usize> {
    TEXT.iter().position(|n| *n == text)
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

    fn draw_tile(&self, t: &Texture2D, index: usize, x: i32, y: i32) {
        let tile_size = 48.0;
        let params = DrawTextureParams {
            source: Some(Rect {
                x: 0.,
                y: (index * TILE_SIZE) as f32,
                w: TILE_SIZE as f32,
                h: TILE_SIZE as f32,
            }),
            dest_size: Some(Vec2::new(tile_size, tile_size)),
            ..Default::default()
        };
        draw_texture_ex(t, x as f32 * tile_size, y as f32 * tile_size, WHITE, params);
    }

    fn get_pressed_direction() -> Option<Direction> {
        for (key, direction) in KEYMAP {
            if is_key_pressed(*key) {
                return Some(*direction);
            }
        }
        None
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

    pub async fn draw(&self) {
        clear_background(BLACK);
        draw_text(format!("{}", self.level.name()), 0.0, 20.0, 30.0, LIGHTGRAY);

        for x in 0..self.level.grid().width() {
            for y in 0..self.level.grid().height() {
                let cell = self.level.grid().at(Pos::new(x, y));
                let units = cell.units();
                let Some(unit) = units.first() else { continue };
                match unit.kind() {
                    UnitKind::Object(noun) => {
                        if let Some(src) = noun_index(noun) {
                            self.draw_tile(&self.sprites, src, x, y);
                        }
                    }
                    UnitKind::Text(text) => {
                        if let Some(src) = text_index(text) {
                            self.draw_tile(&self.words, src, x, y);
                        }
                    }
                }
            }
        }

        next_frame().await
    }
}

pub async fn run_game(level_src: &str) {
    let level = Level::read(level_src);
    let mut viewer = Viewer::new(level).await;
    loop {
        viewer.update();
        viewer.draw().await;
        if *viewer.status() == TurnResult::Win {
            println!("You win!");
            break;
        }
    }
}
