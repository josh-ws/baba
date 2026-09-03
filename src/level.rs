use crate::{
    eval::{Turn, TurnResult},
    world::{Direction, Grid},
};

#[derive(Debug)]
pub struct Level {
    name: String,
    grid: Grid,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            name: Default::default(),
            grid: Grid::empty(0, 0),
        }
    }
}

impl Level {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn read(src: &str) -> Level {
        let mut level = Level::new();
        let mut data = String::new();
        for line in src.lines() {
            match line.trim().split_once("=") {
                Some((key, value)) => match key.trim() {
                    "Name" => level.name = value.trim().to_string(),
                    _ => (),
                },
                None => {
                    data.push_str(line);
                    data.push('\n');
                }
            }
        }
        level.grid = Grid::from_ascii(&data);
        level
    }

    pub fn update(&mut self, dir: Direction) -> TurnResult {
        Turn::new(&mut self.grid, dir).run()
    }
}
