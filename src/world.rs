use crate::unit::{Text, Unit, UnitKind};

#[derive(Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn as_step(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn shift(self, dir: Direction) -> Self {
        let step = dir.as_step();
        Self {
            x: self.x + step.0,
            y: self.y + step.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    units: Vec<Unit>,
}

impl Cell {
    fn new() -> Self {
        Self { units: Vec::new() }
    }

    pub fn first_word(&self) -> Option<Text> {
        self.units
            .iter()
            .filter_map(|f| match f.kind() {
                UnitKind::Object(_) => None,
                UnitKind::Text(t) => Some(t),
            })
            .next_back()
    }

    pub fn units(&self) -> &Vec<Unit> {
        &self.units
    }

    pub fn units_mut(&mut self) -> &mut Vec<Unit> {
        &mut self.units
    }
}

#[derive(Debug, PartialEq)]
pub struct Grid {
    cells: Vec<Cell>,
    w: i32,
    h: i32,
    next_id: u64,
}

impl Grid {
    pub fn empty(w: i32, h: i32) -> Self {
        Self {
            w,
            h,
            cells: vec![Cell::new(); (w * h) as usize],
            next_id: 0,
        }
    }

    pub fn cells(&self) -> &Vec<Cell> {
        &self.cells
    }

    pub fn next(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn iter(&self, from: Pos, dir: Direction) -> Vec<Pos> {
        let mut points = Vec::new();
        let mut pos = from;
        while self.in_bounds(pos) {
            points.push(pos);
            pos = pos.shift(dir);
        }
        points
    }

    pub fn at(&self, pos: Pos) -> &Cell {
        let index = (pos.y * self.width() + pos.x) as usize;
        &self.cells[index]
    }

    pub fn at_mut(&mut self, pos: Pos) -> &mut Cell {
        let index = (pos.y * self.width() + pos.x) as usize;
        &mut self.cells[index]
    }

    pub fn in_bounds(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.x < self.w && pos.y >= 0 && pos.y < self.h
    }

    pub fn width(&self) -> i32 {
        self.w
    }

    pub fn height(&self) -> i32 {
        self.h
    }
}
