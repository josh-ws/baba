use baba::scenes::{Scene, Transition, level::LevelScene};
use macroquad::texture::{FilterMode, Texture2D, load_texture};

use crate::views::level::LevelView;

enum ActiveScene {
    Level(LevelScene, LevelView),
}

impl ActiveScene {
    fn update(&mut self) -> Option<Transition> {
        match self {
            ActiveScene::Level(scene, viewer) => {
                let events = viewer.input().map(|a| scene.update(a));
                viewer.update(scene, &events.unwrap_or_default());
            }
        }
        None
    }

    fn draw(&self) {
        match self {
            ActiveScene::Level(scene, viewer) => {
                viewer.draw(scene);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Resources {
    pub font: Texture2D,
    pub particles: Texture2D,
    pub sprites: Texture2D,
    pub tiles: Texture2D,
    pub words: Texture2D,
}

impl Resources {
    pub async fn load() -> Result<Self, String> {
        Ok(Self {
            font: Self::load_single("assets/font.png").await?,
            particles: Self::load_single("assets/particles.png").await?,
            sprites: Self::load_single("assets/sprites.png").await?,
            tiles: Self::load_single("assets/tiles.png").await?,
            words: Self::load_single("assets/words.png").await?,
        })
    }

    async fn load_single(path: &str) -> Result<Texture2D, String> {
        let texture = load_texture(path)
            .await
            .map_err(|e| format!("could not load texture {path}: {e}"))?;
        texture.set_filter(FilterMode::Nearest);
        Ok(texture)
    }
}

pub struct Game {
    resources: Resources,
    stack: Vec<ActiveScene>,
}

impl Game {
    pub fn new(resources: Resources, initial_scene: Scene) -> Self {
        let mut game = Self {
            resources,
            stack: vec![],
        };
        game.push(initial_scene);
        game
    }

    fn push(&mut self, scene: Scene) {
        let active = match scene {
            Scene::Level(scene) => ActiveScene::Level(scene, LevelView::new(&self.resources)),
        };
        self.stack.push(active);
    }

    // TODO(jw) TEMP. The levelpack should live on the Game, this should not reach into the state
    pub fn reload(&mut self, path: &str) {
        if let Some(ActiveScene::Level(scene, _)) = self.stack.last_mut()
            && let Err(err) = scene.reload(path)
        {
            eprintln!("could not reload pack: {err}");
        }
    }

    pub fn update(&mut self) {
        if let Some(transition) = self.stack.last_mut().and_then(|s| s.update()) {
            println!("{transition:?}");
        }
    }

    pub fn draw(&self) {
        if let Some(s) = self.stack.last() {
            s.draw();
        }
    }
}
