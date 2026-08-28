use crate::{
    lex::lex,
    world::{Direction, Pos},
};

mod ascii;
mod lex;
mod world;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Noun {
    Baba,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operator {
    Is,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Property {
    You,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Text {
    Noun(Noun),
    Operator(Operator),
    Property(Property),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitKind {
    Object(Noun),
    Text(Text),
}

#[derive(Clone, Debug, PartialEq)]
struct Unit {
    id: u64,
    kind: UnitKind,
}

impl Unit {
    fn new(id: u64, kind: UnitKind) -> Self {
        Self { id, kind }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Cell {
    units: Vec<Unit>,
}

impl Cell {
    fn new() -> Self {
        Self { units: Vec::new() }
    }

    fn first_word(&self) -> Option<Text> {
        self.units
            .iter()
            .filter_map(|f| match f.kind {
                UnitKind::Object(_) => None,
                UnitKind::Text(t) => Some(t),
            })
            .last()
    }
}

#[derive(Debug, PartialEq)]
struct Grid {
    cells: Vec<Cell>,
    w: i32,
    h: i32,
    next_id: u64,
}

impl Grid {
    fn empty(w: i32, h: i32) -> Self {
        Self {
            w,
            h,
            cells: vec![Cell::new(); (w * h) as usize],
            next_id: 0,
        }
    }

    fn next(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn iter(&self, from: Pos, dir: Direction) -> Vec<Pos> {
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

fn main() {
    let grid = Grid::from_ascii("BA IS YO BA IS YO");
    let tokens = lex(&grid);
    for token in tokens {
        println!("{}", token.to_ascii());
    }
}
