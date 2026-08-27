enum Direction {
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

struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    pub fn shift(self, dir: Direction) -> Self {
        let step = dir.as_step();
        Self {
            x: self.x + step.0,
            y: self.y + step.1,
        }
    }
}
