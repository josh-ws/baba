use crate::scenes::level::LevelScene;

pub mod level;

#[derive(Debug)]
pub enum Scene {
    Level(LevelScene),
}
