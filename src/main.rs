mod ascii;

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

#[derive(Clone, Debug)]
struct Unit {
    id: u64,
    kind: UnitKind,
}

impl Unit {
    fn new(id: u64, kind: UnitKind) -> Self {
        Self { id, kind }
    }
}

#[derive(Clone, Debug)]
struct Cell {
    units: Vec<Unit>,
}

impl Cell {
    fn new() -> Self {
        Self { units: Vec::new() }
    }

    fn from_units(units: Vec<Unit>) -> Self {
        Self { units }
    }
}

#[derive(Debug)]
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
}

struct World {
    grid: Grid,
}

fn main() {
    let grid = Grid::from_ascii("BA IS YO .. ba");
    println!("{:?}", grid);
    println!("{}", grid.to_ascii())
}
