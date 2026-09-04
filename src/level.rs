use std::collections::HashMap;

use crate::{
    eval::{Turn, TurnResult},
    world::{Direction, Grid, Pos},
};

#[derive(Debug)]
pub enum LevelKind {
    Puzzle,
    Map,
}

impl From<&str> for LevelKind {
    fn from(value: &str) -> Self {
        match &*value.to_ascii_lowercase() {
            "puzzle" => LevelKind::Puzzle,
            "map" => LevelKind::Map,
            _ => panic!("unrecognized level kind {value}"),
        }
    }
}

#[derive(Debug)]
pub struct Level {
    name: String,
    grid: Grid,
    kind: LevelKind,
    links: HashMap<u64, String>,
    grid_history: Vec<Grid>,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            name: Default::default(),
            grid: Grid::empty(0, 0),
            kind: LevelKind::Puzzle,
            links: HashMap::new(),
            grid_history: vec![],
        }
    }
}

impl Level {
    pub fn new() -> Self {
        Self { ..Default::default() }
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
        let mut links = Vec::new();
        for line in src.lines() {
            match line.trim().split_once("=") {
                Some((key, value)) => match key.trim() {
                    "Name" => level.name = value.trim().to_string(),
                    "Kind" => level.kind = value.trim().into(),
                    "Data" => (),
                    "Link" => links.push(value.trim()),
                    _ => panic!("unrecognised key `{}` in level `{}`", key, level.name),
                },
                None => {
                    data.push_str(line);
                    data.push('\n');
                }
            }
        }
        level.grid = Grid::from_ascii(&data);
        for link in links {
            level.parse_link(link);
        }
        level
    }

    pub fn update(&mut self, dir: Direction) -> TurnResult {
        let before = self.grid.clone();
        let result = Turn::new(&mut self.grid, dir).run();
        if before != self.grid {
            self.grid_history.push(before);
        }
        result
    }

    pub fn undo(&mut self) -> bool {
        match self.grid_history.pop() {
            Some(grid) => {
                self.grid = grid;
                true
            }
            None => false,
        }
    }

    pub fn link_for(&self, selected: &[u64]) -> Option<&str> {
        for unit in selected {
            if let Some(key) = self.links.get(unit) {
                return Some(key);
            }
        }
        None
    }

    // TODO(jw) this sucks
    fn parse_link(&mut self, src: &str) {
        let (pos, key) = src
            .split_once(" ")
            .and_then(|(position, key)| match position.split_once(",") {
                Some((x, y)) => Some((x, y, key)),
                None => None,
            })
            .and_then(|(x, y, key)| {
                let ix = x.parse::<i32>().expect("x component in invalid format");
                let iy = y.parse::<i32>().expect("y component in invalid format");
                Some((Pos::new(ix, iy), key))
            })
            .expect("invalid link format: should be `x,y level`");
        if let Some(unit) = self.grid.at(pos).units().last() {
            self.links.insert(unit.id(), key.to_string());
        }
    }
}
