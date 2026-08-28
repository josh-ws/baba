use crate::{
    lex::lex,
    rule::{Rules, parse},
    unit::Property,
    world::{Cell, Direction, Grid},
};

pub enum Result {
    Continue,
}

pub struct Turn<'a> {
    grid: &'a mut Grid,
    rules: Rules,
    input: Direction,
}

impl<'a> Turn<'a> {
    pub fn new(grid: &'a mut Grid, input: Direction) -> Self {
        Self {
            grid,
            input,
            rules: Rules::new(),
        }
    }

    pub fn run(&mut self) -> Result {
        self.reparse();
        self.move_you();
        self.reparse();
        self.handle_transforms();
        self.reparse();
        self.check_status()
    }

    pub fn grid(&mut self) -> &mut Grid {
        self.grid
    }

    fn check_status(&self) -> Result {
        Result::Continue
    }

    fn reparse(&mut self) {
        self.rules = parse(&lex(self.grid));
    }

    fn move_you(&mut self) {
        let you = query_prop(self.grid, &self.rules, Property::You);
        for id in you {
            if let Some(from) = self.grid().find_unit(id) {
                let to = from.shift(self.input);
                if self.grid.in_bounds(to)
                    && !cell_has(self.grid.at(to), &self.rules, Property::Stop)
                {
                    self.grid.move_unit(id, from, to);
                }
            }
        }
    }

    fn handle_transforms(&mut self) {}
}

// returns all noun units with specified property
fn query_prop(grid: &Grid, rules: &Rules, prop: Property) -> Vec<u64> {
    grid.cells()
        .iter()
        .flat_map(Cell::units)
        .filter(|u| rules.has(u.noun(), prop))
        .map(|u| u.id())
        .collect::<Vec<u64>>()
}

fn cell_has(cell: &Cell, rules: &Rules, prop: Property) -> bool {
    cell.units().iter().any(|u| rules.has(u.noun(), prop))
}

#[cfg(test)]
mod tests {
    use crate::{
        eval::Turn,
        world::{Direction, Grid},
    };

    #[test]
    fn basic_move() {
        let mut grid = Grid::from_ascii("BA IS YO ba .. ..");
        let mut turn = Turn::new(&mut grid, Direction::East);
        turn.run();
        let result = grid.to_ascii();
        assert_eq!("BA IS YO .. ba ..", result);
    }

    #[test]
    fn move_blocked_on_edge() {
        let mut grid = Grid::from_ascii("BA IS YO .. .. ba");
        Turn::new(&mut grid, Direction::East).run();
        assert_eq!("BA IS YO .. .. ba", grid.to_ascii());
    }

    #[test]
    fn move_does_not_wrap() {
        let mut grid = Grid::from_ascii("BA IS YO\n.. .. ba\n.. .. ..");
        Turn::new(&mut grid, Direction::East).run();
        assert_eq!("BA IS YO\n.. .. ba\n.. .. ..", grid.to_ascii());
    }

    #[test]
    fn move_blocked_by_stop() {
        let mut grid = Grid::from_ascii("BA IS YO .. ba wa .. WA IS ST");
        Turn::new(&mut grid, Direction::East).run();
        assert_eq!("BA IS YO .. ba wa .. WA IS ST", grid.to_ascii());
    }

    #[test]
    fn move_not_blocked_when_no_stop() {
        let mut grid = Grid::from_ascii("BA IS YO .. ba wa");
        Turn::new(&mut grid, Direction::East).run();
        assert_eq!("BA IS YO .. .. ba", grid.to_ascii());
    }
}
