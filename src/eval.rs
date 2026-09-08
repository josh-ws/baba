use crate::{
    lex::lex,
    query::{
        UnitRef, any_cell_has, cell_has, query_defeat, query_has, query_is_noun, query_is_property, query_selected,
        query_sink,
    },
    rule::{Rules, parse},
    unit::{
        Property::{self},
        UnitKind,
    },
    world::{Direction, Grid, Pos},
};

#[derive(Debug, PartialEq)]
pub enum TurnStatus {
    Continue,
    Win,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Cause {
    Sink,
    Defeat,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Destroyed { pos: Pos, cause: Cause },
}

#[derive(Debug, PartialEq)]
pub struct TurnResult {
    pub status: TurnStatus,
    pub selected: Vec<u64>, // all unit ids that a SELECT unit is touching
    pub events: Vec<Event>,
}

pub struct Turn<'a> {
    grid: &'a mut Grid,
    rules: Rules,
    input: Direction,
    events: Vec<Event>,
}

impl<'a> Turn<'a> {
    pub fn new(grid: &'a mut Grid, input: Direction) -> Self {
        Self {
            grid,
            input,
            rules: Rules::new(),
            events: Vec::new(),
        }
    }

    pub fn run(mut self) -> TurnResult {
        self.reparse();
        self.move_you().then(|| self.reparse());
        self.move_select().then(|| self.reparse());
        self.handle_transforms().then(|| self.reparse());
        self.handle_sink().then(|| self.reparse());
        self.handle_defeats().then(|| self.reparse());
        TurnResult {
            status: self.check_status(),
            selected: query_selected(&self.rules, self.grid)
                .iter()
                .map(|unit_ref| unit_ref.unit_id)
                .collect::<Vec<u64>>(),
            events: self.events,
        }
    }

    fn check_status(&self) -> TurnStatus {
        if any_cell_has(&self.rules, self.grid, &[Property::Win, Property::You]) {
            TurnStatus::Win
        } else {
            TurnStatus::Continue
        }
    }

    fn reparse(&mut self) {
        self.rules = parse(&lex(self.grid));
    }

    fn move_you(&mut self) -> bool {
        let you = query_is_property(&self.rules, self.grid, Property::You);
        let mut moved = false;
        for unit in &you {
            if let Some((from, _)) = self.grid.find_unit(unit.unit_id) {
                moved |= push(self.grid, &self.rules, unit.unit_id, from, self.input);
            }
        }
        moved
    }

    fn move_select(&mut self) -> bool {
        let select = query_is_property(&self.rules, self.grid, Property::Select);
        let mut moved = false;
        for unit in &select {
            let target = unit.pos.shift(self.input);
            if !self.grid.in_bounds(target) {
                continue;
            }
            if !self.grid.at(target).units().iter().any(|f| f.is_object()) {
                continue;
            }
            self.grid
                .move_matching(unit.pos, target, self.input, |p| p.id() == unit.unit_id);
            moved = true;
        }
        moved
    }

    fn handle_transforms(&mut self) -> bool {
        let transforms = query_is_noun(&self.rules, self.grid);
        for t in &transforms {
            self.grid.transform_unit(t.unit_id, t.pos, t.into_noun);
        }

        !transforms.is_empty()
    }

    fn handle_sink(&mut self) -> bool {
        let sinks = query_sink(&self.rules, self.grid);
        self.destroy_and_create(&sinks, Cause::Sink)
    }

    fn handle_defeats(&mut self) -> bool {
        let defeated = query_defeat(&self.rules, self.grid);
        self.destroy_and_create(&defeated, Cause::Defeat)
    }

    // destroy all units in `doomed` and resolve their HAS rules.
    fn destroy_and_create(&mut self, doomed: &[UnitRef], cause: Cause) -> bool {
        let mut changed = false;
        let ids = doomed.iter().map(|u| u.unit_id).collect::<Vec<u64>>();
        let spawns = query_has(&self.rules, self.grid, &ids);
        for target in doomed {
            if self.grid.destroy_unit(target.unit_id) {
                changed = true;
                self.events.push(Event::Destroyed { pos: target.pos, cause });
            }
        }
        for new in &spawns {
            self.grid.create_unit(new.pos, UnitKind::Object(new.into_noun));
        }
        changed || !spawns.is_empty()
    }
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
        if cell_has(rules, grid.at(next), &[Property::Stop]) {
            return None;
        }
        if !cell_has(rules, grid.at(next), &[Property::Push]) {
            return Some(cells_to_move);
        }
        cells_to_move.push(next);
        next = next.shift(dir); // move to next cell
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        eval::{Cause, Event, Turn, TurnStatus},
        world::{Direction, Grid, Pos},
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

    #[test]
    fn sink_reports_destroy_event() {
        let mut grid = Grid::from_ascii("BA IS YO WT IS SI ba wt");
        let result = Turn::new(&mut grid, Direction::East).run();
        assert_eq!(
            result.events,
            vec![
                Event::Destroyed {
                    pos: Pos::new(7, 0),
                    cause: Cause::Sink,
                },
                Event::Destroyed {
                    pos: Pos::new(7, 0),
                    cause: Cause::Sink,
                }
            ]
        )
    }

    #[test]
    fn defeat_reports_destroy_event() {
        let mut grid = Grid::from_ascii("BA IS YO WT IS DE ba wt");
        let result = Turn::new(&mut grid, Direction::East).run();
        assert_eq!(
            result.events,
            vec![Event::Destroyed {
                pos: Pos::new(7, 0),
                cause: Cause::Defeat,
            },]
        )
    }
}
