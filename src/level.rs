use std::collections::HashMap;

use crate::{
    eval::{Turn, TurnResult},
    world::{Direction, Grid, Pos},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelKind {
    Puzzle,
    Map,
}

impl TryFrom<&str> for LevelKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match &*value.to_ascii_lowercase() {
            "puzzle" => Ok(LevelKind::Puzzle),
            "map" => Ok(LevelKind::Map),
            _ => Err(format!("unrecognized level kind {value}")),
        }
    }
}

#[derive(Debug)]
pub struct Link {
    key: String,
    display: String,
}

impl Link {
    fn new(key: String, display: String) -> Self {
        Self { key, display }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn display(&self) -> &str {
        &self.display
    }
}

#[derive(Debug)]
pub struct Level {
    name: String,
    grid: Grid,
    kind: LevelKind,
    links: HashMap<u64, Link>,
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

    pub fn links(&self) -> &HashMap<u64, Link> {
        &self.links
    }

    pub fn kind(&self) -> LevelKind {
        self.kind
    }

    pub fn read(src: &str) -> Result<Level, String> {
        let mut level = Level::new();
        let mut data = String::new();
        let mut links = Vec::new();
        for line in src.lines() {
            match line.trim().split_once("=") {
                Some((key, value)) => match key.trim() {
                    "Name" => level.name = value.trim().to_string(),
                    "Kind" => level.kind = value.trim().try_into()?,
                    "Data" => (),
                    "Link" => links.push(value.trim()),
                    _ => return Err(format!("unrecognised key `{}` in level `{}`", key, level.name)),
                },
                None => {
                    data.push_str(line);
                    data.push('\n');
                }
            }
        }
        level.grid = Grid::try_from_ascii(&data)?;
        for link in links {
            level.parse_link(link)?;
        }
        Ok(level)
    }

    pub fn update(&mut self, dir: Option<Direction>) -> TurnResult {
        let (grid, result) = Turn::new(self.grid.clone(), dir).run();
        if grid != self.grid {
            self.grid_history.push(std::mem::replace(&mut self.grid, grid));
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

    pub fn link_for(&self, selected: &[u64]) -> Option<&Link> {
        for unit in selected {
            if let Some(key) = self.links.get(unit) {
                return Some(key);
            }
        }
        None
    }

    fn parse_link(&mut self, src: &str) -> Result<(), String> {
        let parts = src.split(",").collect::<Vec<&str>>();
        let (x, y, key, display) = match &parts[..] {
            [x, y, key] => (x, y, key, &"A"),
            [x, y, key, display] => (x, y, key, display),
            _ => return Err("link {src} in invalid format: should be `x,y,key,display`".to_string()),
        };

        let ix = x.parse::<i32>().map_err(|_| format!("link {src} X position invalid"))?;
        let iy = y.parse::<i32>().map_err(|_| format!("link {src} Y position invalid"))?;
        let pos = Pos::new(ix, iy);
        if !self.grid.in_bounds(pos) {
            return Err(format!("link coordinates out of bounds: `{src}` at {ix},{iy}"));
        }
        let unit = self
            .grid
            .at(pos)
            .units()
            .last()
            .ok_or_else(|| format!("link {src} links to empty or invalid unit"))?;

        let link = Link::new(key.to_string(), display.to_string());
        self.links.insert(unit.id(), link);
        Ok(())
    }
}
