use baba::scenes::{Scene, level::LevelScene};
use macroquad::{
    rand::rand,
    texture::{FilterMode, Texture2D, load_texture},
};

use crate::{
    render::transition::{Transition, TransitionSmoothing, TransitionStyle},
    views::level::LevelView,
};

enum ActiveScene {
    Level(LevelScene, LevelView),
}

impl ActiveScene {
    fn update(&mut self, accept_input: bool) {
        match self {
            ActiveScene::Level(scene, viewer) => {
                let act = if accept_input { viewer.input() } else { None };
                let events = act.map(|a| scene.update(a));
                viewer.update(scene, &events.unwrap_or_default());
            }
        }
    }

    fn draw(&self) {
        match self {
            ActiveScene::Level(scene, viewer) => {
                viewer.draw(scene);
            }
        }
    }

    fn is_leaving(&self) -> bool {
        match self {
            ActiveScene::Level(scene, _) => scene.pending().is_some(),
        }
    }

    fn commit(&mut self) {
        match self {
            ActiveScene::Level(scene, _) => scene.commit(),
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
    transition: Option<Transition>,
}

impl Game {
    pub fn new(resources: Resources, initial_scene: Scene) -> Self {
        let mut game = Self {
            resources,
            stack: vec![],
            transition: None,
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
        let Some(top) = self.stack.last_mut() else { return };
        top.update(self.transition.is_none());

        if let Some(transition) = &mut self.transition {
            if transition.update() {
                top.commit();
            }
            if !transition.is_active() {
                self.transition = None;
            }
        } else if top.is_leaving() {
            let transitions = [
                TransitionStyle::Diamond,
                TransitionStyle::Fade,
                TransitionStyle::Interlaced,
            ];
            let i = (rand() as usize) % transitions.len();
            self.transition = Some(Transition::new(transitions[i], TransitionSmoothing::Smoothstep));
        }
    }

    pub fn draw(&self) {
        if let Some(s) = self.stack.last() {
            s.draw();
        }
        if let Some(transition) = &self.transition {
            transition.draw();
        }
    }
}
