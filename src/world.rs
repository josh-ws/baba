use crate::unit::{Noun, Text, Unit, UnitKind};

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Clone)]
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

    pub fn next(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn iter(&self, from: Pos, dir: Direction) -> impl Iterator<Item = Pos> {
        std::iter::successors(Some(from), move |p| Some(p.shift(dir))).take_while(|p| self.in_bounds(*p))
    }

    pub fn at(&self, pos: Pos) -> &Cell {
        debug_assert!(self.in_bounds(pos));
        let index = (pos.y * self.width() + pos.x) as usize;
        &self.cells[index]
    }

    pub fn at_mut(&mut self, pos: Pos) -> &mut Cell {
        debug_assert!(self.in_bounds(pos));
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

    fn pos_of(&self, i: usize) -> Pos {
        Pos::new(i as i32 % self.w, i as i32 / self.w)
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn cells_with_pos(&self) -> impl Iterator<Item = (Pos, &Cell)> {
        self.cells.iter().enumerate().map(|(i, c)| (self.pos_of(i), c))
    }

    pub fn units(&self) -> impl Iterator<Item = &Unit> {
        self.units_with_pos().map(|(_, u)| u)
    }

    pub fn units_with_pos(&self) -> impl Iterator<Item = (Pos, &Unit)> {
        self.cells_with_pos()
            .flat_map(|(pos, c)| c.units().iter().map(move |u| (pos, u)))
    }

    pub fn find_unit(&self, id: u64) -> Option<(Pos, &Unit)> {
        self.units_with_pos().find(|(_, unit)| unit.id() == id)
    }

    pub fn create_unit(&mut self, pos: Pos, kind: UnitKind) {
        let unit = Unit::new(self.next(), kind);
        self.at_mut(pos).units_mut().push(unit);
    }

    pub fn destroy_unit(&mut self, id: u64) -> bool {
        if let Some((pos, _)) = self.find_unit(id) {
            self.at_mut(pos).units_mut().retain(|u| u.id() != id);
            true
        } else {
            false
        }
    }

    // moves all units on `from` to `to`, that match `p`
    pub fn move_matching(&mut self, from: Pos, to: Pos, dir: Direction, p: impl Fn(&Unit) -> bool) {
        let mut units = self
            .at_mut(from)
            .units_mut()
            .extract_if(.., |u| p(u))
            .collect::<Vec<Unit>>();
        for unit in &mut units {
            unit.set_direction(dir);
        }
        self.at_mut(to).units_mut().extend(units);
    }

    pub fn transform_unit(&mut self, id: u64, pos: Pos, into: Noun) {
        let Some(i) = self.at(pos).units().iter().position(|u| u.id() == id) else {
            return;
        };
        let new_id = self.next();
        self.at_mut(pos).units_mut()[i] = Unit::new(new_id, UnitKind::Object(into));
    }
}
