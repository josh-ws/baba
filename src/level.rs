use crate::{
    eval::{Turn, TurnResult},
    world::{Direction, Grid},
};

#[derive(Debug)]
pub struct Level {
    key: String,
    name: String,
    grid: Grid,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            key: Default::default(),
            name: Default::default(),
            grid: Grid::empty(0, 0),
        }
    }
}

impl Level {
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn read(src: &str) -> Level {
        let mut level = Level::default();
        let mut data = String::new();
        for line in src.lines() {
            match line.trim().split_once("=") {
                Some((key, value)) => match key.trim() {
                    "Key" => level.key = value.trim().to_string(),
                    "Name" => level.name = value.trim().to_string(),
                    _ => (),
                },
                None => data.push_str(&*format!("{line}\n")),
            }
        }
        level.grid = Grid::from_ascii(&data);
        level
    }

    pub fn update(&mut self, dir: Direction) -> TurnResult {
        Turn::new(&mut self.grid, dir).run()
    }
}
