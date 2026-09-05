use crate::{
    lex::lex,
    rule::{Complement, Rules, parse},
    unit::{
        Noun, Operator,
        Property::{self},
        Unit, UnitKind,
    },
    world::{Cell, Direction, Grid, Pos},
};

#[derive(Debug, PartialEq)]
pub enum TurnStatus {
    Continue,
    Win,
}

#[derive(Debug, PartialEq)]
pub struct TurnResult {
    pub status: TurnStatus,
    pub selected: Vec<u64>, // all unit ids that a SELECT unit is touching
}

impl TurnResult {
    pub fn new(status: TurnStatus, selected: Vec<u64>) -> Self {
        Self { status, selected }
    }
}

struct Transformation {
    unit_id: u64,
    pos: Pos,
    into: Noun,
}

impl Transformation {
    fn new(unit_id: u64, pos: Pos, into: Noun) -> Self {
        Transformation { unit_id, pos, into }
    }
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

    pub fn run(&mut self) -> TurnResult {
        self.reparse();
        self.move_you().then(|| self.reparse());
        self.move_select().then(|| self.reparse());
        self.handle_transforms().then(|| self.reparse());
        self.handle_sink().then(|| self.reparse());
        self.handle_defeats().then(|| self.reparse());
        TurnResult::new(self.check_status(), query_selected(self.grid, &self.rules))
    }

    fn check_status(&self) -> TurnStatus {
        if any_cell_has(self.grid, &self.rules, &[Property::Win, Property::You]) {
            TurnStatus::Win
        } else {
            TurnStatus::Continue
        }
    }

    fn reparse(&mut self) {
        self.rules = parse(&lex(self.grid));
    }

    fn move_you(&mut self) -> bool {
        let you = query_prop(self.grid, &self.rules, Property::You);
        let mut moved = false;
        for id in &you {
            if let Some((from, _)) = self.grid.find_unit(*id) {
                moved |= push(self.grid, &self.rules, *id, from, self.input);
            }
        }
        moved
    }

    fn move_select(&mut self) -> bool {
        let select = query_prop(self.grid, &self.rules, Property::Select);
        let mut moved = false;
        for id in &select {
            if let Some((from, _)) = self.grid.find_unit(*id) {
                let target = from.shift(self.input);
                if !self.grid.in_bounds(target) {
                    continue;
                }
                if !self.grid.at(target).units().iter().any(|f| f.is_object()) {
                    continue;
                }
                self.grid.move_matching(from, target, self.input, |p| p.id() == *id);
                moved = true;
            }
        }
        moved
    }

    fn handle_transforms(&mut self) -> bool {
        let transforms = query_transforms(self.grid, &self.rules);
        for t in &transforms {
            self.grid.transform_unit(t.unit_id, t.pos, t.into);
        }

        !transforms.is_empty()
    }

    fn handle_sink(&mut self) -> bool {
        let sinks = query_sinks(self.grid, &self.rules);
        self.destroy_and_create(&sinks)
    }

    fn handle_defeats(&mut self) -> bool {
        let defeated = query_defeats(self.grid, &self.rules);
        self.destroy_and_create(&defeated)
    }

    // destroy all units in `doomed` and resolve their HAS rules.
    fn destroy_and_create(&mut self, doomed: &[u64]) -> bool {
        let mut changed = false;
        let spawns = query_has(self.grid, &self.rules, doomed);
        for target in doomed {
            changed |= self.grid.destroy_unit(*target);
        }
        for (pos, noun) in &spawns {
            self.grid.create_unit(*pos, UnitKind::Object(*noun));
        }
        changed || !spawns.is_empty()
    }
}

// query the grid, assuming all units in `doomed` will be destroyed.
// if A HAS B and A is in `doomed`, then `B` is returned with the new position.
fn query_has(grid: &Grid, rules: &Rules, doomed: &[u64]) -> Vec<(Pos, Noun)> {
    let mut result = Vec::new();
    for unit in doomed {
        if let Some((pos, unit)) = grid.find_unit(*unit) {
            let create = rules.unit_has(unit.noun());
            for created_noun in create {
                result.push((pos, created_noun));
            }
        }
    }
    result
}

fn query_transforms(grid: &Grid, rules: &Rules) -> Vec<Transformation> {
    let mut result = Vec::new();
    for (pos, unit) in grid.units_with_pos() {
        if let Some(target) = transforming_into(rules, unit.noun()) {
            result.push(Transformation::new(unit.id(), pos, target));
        }
    }
    result
}

fn query_sinks(grid: &Grid, rules: &Rules) -> Vec<u64> {
    grid.cells()
        .iter()
        .filter(|c| cell_has(c, rules, Property::Sink))
        .filter(|c| c.units().iter().count() > 1)
        .flat_map(|c| c.units())
        .map(Unit::id)
        .collect::<Vec<u64>>()
}

fn query_defeats(grid: &Grid, rules: &Rules) -> Vec<u64> {
    grid.cells()
        .iter()
        .filter(|c| cell_has(c, rules, Property::Defeat))
        .flat_map(|c| c.units())
        .filter(|unit| rules.unit_has_prop(unit.noun(), Property::You))
        .map(|u| u.id())
        .collect::<Vec<u64>>()
}

fn transforming_into(rules: &Rules, noun: Noun) -> Option<Noun> {
    let targets = rules
        .iter()
        .filter_map(|rule| match rule.complement {
            Complement::Noun(t) => {
                if rule.subject == noun && rule.operator == Operator::Is {
                    Some(t)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<Noun>>();

    if targets.contains(&noun) {
        return None;
    }
    targets.first().copied() // TODO(jw) explicitly returning the first transformation here, we should handle all of them
}

// returns all noun units with specified property
fn query_prop(grid: &Grid, rules: &Rules, prop: Property) -> Vec<u64> {
    grid.units()
        .filter(|u| rules.unit_has_prop(u.noun(), prop))
        .map(|u| u.id())
        .collect::<Vec<u64>>()
}

fn query_selected(grid: &Grid, rules: &Rules) -> Vec<u64> {
    grid.cells()
        .iter()
        .filter(|c| cell_has(c, rules, Property::Select))
        .flat_map(|c| c.units())
        .filter(|u| u.is_object() && !rules.unit_has_prop(u.noun(), Property::Select))
        .map(Unit::id)
        .collect::<Vec<u64>>()
}

fn cell_has(cell: &Cell, rules: &Rules, prop: Property) -> bool {
    cell.units().iter().any(|u| rules.unit_has_prop(u.noun(), prop))
}

// check whether the grid has any cell that satisfies all props in `props`
fn any_cell_has(grid: &Grid, rules: &Rules, props: &[Property]) -> bool {
    grid.cells()
        .iter()
        .any(|c| props.iter().all(|p| cell_has(c, rules, *p)))
}

fn push(grid: &mut Grid, rules: &Rules, mover: u64, from: Pos, dir: Direction) -> bool {
    match movement_chain(grid, rules, from, dir) {
        Some(cells) => {
            for (i, pos) in cells.iter().enumerate().rev() {
                let to = pos.shift(dir);
                if i == 0 {
                    grid.move_matching(*pos, to, dir, |u| u.id() == mover);
                } else {
                    grid.move_matching(*pos, to, dir, |u| rules.unit_has_prop(u.noun(), Property::Push));
                }
            }
            true
        }
        None => false,
    }
}

// walk the grid from `from` in direction `dir`, collecting all cells that must move
// `None` result means movement is impossible
fn movement_chain(grid: &Grid, rules: &Rules, from: Pos, dir: Direction) -> Option<Vec<Pos>> {
    let mut cells_to_move = vec![from];
    let mut next = from.shift(dir);
    loop {
        if !grid.in_bounds(next) {
            return None;
        }
        if cell_has(grid.at(next), rules, Property::Stop) {
            return None;
        }
        if !cell_has(grid.at(next), rules, Property::Push) {
            return Some(cells_to_move);
        }
        cells_to_move.push(next);
        next = next.shift(dir); // move to next cell
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        eval::{Turn, TurnStatus},
        world::{Direction, Grid},
    };

    #[track_caller]
    fn assert_move_result(before: &str, after: &str, dir: Direction) {
        let mut grid = Grid::from_ascii(before);
        Turn::new(&mut grid, dir).run();
        assert_eq!(after, grid.to_ascii());
    }

    #[track_caller]
    fn assert_result(src: &str, dir: Direction, expected: TurnStatus) {
        let mut grid = Grid::from_ascii(src);
        let result = Turn::new(&mut grid, dir).run();
        assert_eq!(result.status, expected);
    }

    #[test]
    fn basic_you_movement() {
        assert_move_result("BA IS YO ba .. ..", "BA IS YO .. ba ..", Direction::East);
        assert_move_result("BA IS YO .. ba ..", "BA IS YO ba .. ..", Direction::West);
        assert_move_result("BA IS YO ba\n.. .. .. ..", "BA IS YO ..\n.. .. .. ba", Direction::South);
        assert_move_result("BA IS YO ..\n.. .. .. ba", "BA IS YO ba\n.. .. .. ..", Direction::North);
    }

    #[test]
    fn you_multi_move() {
        assert_move_result("BA IS YO ba ba ba .. ..", "BA IS YO .. ba ba ba ..", Direction::East);
    }

    #[test]
    fn you_blocked_on_edge() {
        assert_move_result("BA IS YO .. .. ba", "BA IS YO .. .. ba", Direction::East);
        assert_move_result("ba .. .. BA IS YO", "ba .. .. BA IS YO", Direction::West);
        assert_move_result("BA IS YO .. ba ..", "BA IS YO .. ba ..", Direction::North);
        assert_move_result("BA IS YO .. ba ..", "BA IS YO .. ba ..", Direction::South);
    }

    #[test]
    fn you_blocked_by_stop() {
        assert_move_result(
            "BA IS YO .. ba wa .. WA IS ST",
            "BA IS YO .. ba wa .. WA IS ST",
            Direction::East,
        );
    }
    #[test]
    fn basic_push() {
        assert_move_result(
            "BA IS YO RO IS PU .. ba ro ..",
            "BA IS YO RO IS PU .. .. ba ro",
            Direction::East,
        );
        assert_move_result(
            "BA IS YO RO IS PU .. ro ba ..",
            "BA IS YO RO IS PU ro ba .. ..",
            Direction::West,
        );
        assert_move_result(
            "BA IS YO ..\nRO IS PU ro\n.. .. .. ba",
            "BA IS YO ro\nRO IS PU ba\n.. .. .. ..",
            Direction::North,
        );
        assert_move_result(
            "BA IS YO ba\nRO IS PU ro\n.. .. .. ..",
            "BA IS YO ..\nRO IS PU ba\n.. .. .. ro",
            Direction::South,
        );
    }

    #[test]
    fn push_chain() {
        assert_move_result(
            "BA IS YO RO IS PU ba ro ro .. ro ..",
            "BA IS YO RO IS PU .. ba ro ro ro ..",
            Direction::East,
        );
    }

    #[test]
    #[ignore = "YOU chain needs to keep track of pushed"]
    fn you_push_chain() {
        assert_move_result(
            "BA IS YO BA IS PU ba ba .. ..",
            "BA IS YO BA IS PU .. ba ba ..",
            Direction::East,
        );
    }

    #[test]
    fn push_blocked_by_stop() {
        assert_move_result(
            "BA IS YO RO IS PU WA IS ST ba ro wa",
            "BA IS YO RO IS PU WA IS ST ba ro wa",
            Direction::East,
        );
    }

    #[test]
    fn win_standing_on_win_tile() {
        assert_result("BA IS YO RO IS WI ba ro", Direction::East, TurnStatus::Win);
    }

    #[test]
    fn win_when_you_is_win() {
        assert_result("BA IS YO BA IS WI ba", Direction::East, TurnStatus::Win);
    }

    #[test]
    fn text_is_pushable() {
        assert_move_result("BA IS YO ba BA ..", "BA IS YO .. ba BA", Direction::East);
    }

    #[test]
    fn grid_is_reevaluated_after_pushing_words() {
        assert_result("BA IS YO ba BA .. IS WI", Direction::East, TurnStatus::Win);
    }

    #[test]
    fn transform() {
        assert_move_result("BA IS RO ba", "BA IS RO ro", Direction::East);
        assert_move_result("BA IS RO IS BA ba ro", "BA IS RO IS BA ro ba", Direction::East);
    }

    #[test]
    fn transform_no_loopback() {
        assert_move_result("BA IS RO IS BA ba", "BA IS RO IS BA ro", Direction::East);
    }

    #[test]
    fn transform_x_is_x() {
        assert_move_result("BA IS RO BA IS BA ba", "BA IS RO BA IS BA ba", Direction::East);
    }

    // test level is stop by default
    #[test]
    fn level_is_stop_inherently() {
        assert_move_result("BA IS YO ba le", "BA IS YO ba le", Direction::East);
        assert_move_result(
            "BA IS YO LE IS PU ba le ..",
            "BA IS YO LE IS PU ba le ..",
            Direction::East,
        );
    }

    #[test]
    fn select_can_move_onto_objects() {
        assert_move_result("cu ba CU IS SE", ".. cu CU IS SE", Direction::East);
    }

    #[test]
    fn select_cannot_move_onto_words() {
        assert_move_result("CU IS SE cu BA", "CU IS SE cu BA", Direction::East);
    }

    #[test]
    fn select_ignores_move_rules() {
        assert_move_result(
            "CU IS SE BA IS PU cu ba ..",
            "CU IS SE BA IS PU .. cu ..",
            Direction::East,
        );
        assert_move_result(
            "CU IS SE BA IS ST cu ba ..",
            "CU IS SE BA IS ST .. cu ..",
            Direction::East,
        );
    }

    #[test]
    fn test_sink_destroys_objects() {
        assert_move_result("WA IS SI BA IS YO ba wa", "WA IS SI BA IS YO .. ..", Direction::East);
        assert_move_result("WA IS SI WA IS YO ba wa", "WA IS SI WA IS YO .. ..", Direction::West);
    }

    #[test]
    fn test_sink_does_not_destroy_self() {
        assert_move_result("WT IS SI wa", "WT IS SI wa", Direction::East);
    }

    #[test]
    fn test_sink_beats_win() {
        assert_result(
            "WT IS SI WT IS WI BA IS YO ba wt",
            Direction::East,
            TurnStatus::Continue,
        );
        assert_move_result(
            "WT IS SI WT IS WI BA IS YO ba wt",
            "WT IS SI WT IS WI BA IS YO .. ..",
            Direction::East,
        );
    }

    #[test]
    fn test_defeat() {
        assert_move_result("BA IS YO RO IS DE ba ro", "BA IS YO RO IS DE .. ro", Direction::East);
    }

    #[test]
    fn test_self_defeat() {
        assert_move_result("BA IS YO BA IS DE ba ..", "BA IS YO BA IS DE .. ..", Direction::East);
    }

    #[test]
    fn test_defeat_beats_win() {
        assert_result(
            "BA IS YO RO IS DE RO IS WI ba ro",
            Direction::East,
            TurnStatus::Continue,
        );
    }

    #[test]
    fn test_defeat_leaves_non_you() {
        assert_move_result(
            "BA IS YO RO IS PU WA IS DE ba ro wa",
            "BA IS YO RO IS PU WA IS DE .. ba ro",
            Direction::East,
        );
    }

    #[test]
    fn test_sink_creates_has_unit() {
        assert_move_result(
            "BA IS YO BA HA RO WT IS SI ba wt",
            "BA IS YO BA HA RO WT IS SI .. ro",
            Direction::East,
        );
    }

    #[test]
    fn test_defeat_creates_has_unit() {
        assert_move_result(
            "BA IS YO BA HA RO WT IS DE ba wt",
            "BA IS YO BA HA RO WT IS DE .. ro",
            Direction::East,
        );
    }
}
