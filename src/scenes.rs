use crate::scenes::level::LevelScene;

pub mod level;

#[derive(Debug)]
pub enum Scene {
    Level(LevelScene),
}

#[derive(Debug)]
pub enum Transition {
    Push(Scene),
    Replace(Scene),
    Pop,
}
