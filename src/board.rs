use crate::{
    eval::{Cause, Event},
    unit::{Noun, UnitKind},
    world::{Direction, Grid, Pos},
};

/// Board is a wrapper around a Grid. Each grid write in a turn goes through here, except
/// for `face()`
pub struct Board {
    grid: Grid,
    log: Vec<Event>,
}

impl Board {
    pub fn new(grid: Grid) -> Self {
        Self { grid, log: Vec::new() }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn step(&mut self, id: u64, dir: Direction) {
        if let Some((from, _)) = self.grid.find_unit(id) {
            let to = from.shift(dir);
            self.grid.move_matching(from, to, dir, |u| u.id() == id);
            self.log.push(Event::Moved { id, from, to });
        }
    }

    pub fn face(&mut self, id: u64, dir: Direction) {
        self.grid.turn_unit(id, dir); // no log yet
    }

    pub fn destroy(&mut self, id: u64, cause: Cause) {
        if let Some((pos, unit)) = self.grid.destroy_unit(id) {
            self.log.push(Event::Destroyed { unit, pos, cause });
        }
    }

    pub fn create(&mut self, pos: Pos, noun: Noun, dir: Direction) {
        self.grid.create_unit(pos, UnitKind::Object(noun), dir);
        self.log.push(Event::Created { pos });
    }

    pub fn transform(&mut self, id: u64, into: &[Noun]) {
        if let Some((pos, _)) = self.grid.find_unit(id) {
            self.grid.transform_unit(id, into);
            for _ in into {
                self.log.push(Event::Created { pos });
            }
        }
    }

    pub fn mark(&self) -> usize {
        self.log.len()
    }

    pub fn finish(self) -> (Grid, Vec<Event>) {
        (self.grid, self.log)
    }

    pub fn since(&self, mark: usize) -> &[Event] {
        &self.log[mark..]
    }
}
