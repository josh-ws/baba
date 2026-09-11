use std::collections::HashSet;

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

struct Movement<'a> {
    grid: &'a mut Grid,
    rules: &'a Rules,
    moved: HashSet<u64>,
}

impl<'a> Movement<'a> {
    fn new(grid: &'a mut Grid, rules: &'a Rules) -> Self {
        Self {
            grid,
            rules,
            moved: HashSet::new(),
        }
    }

    fn turn(&mut self, mover: u64, dir: Direction) {
        self.grid.turn_unit(mover, dir);
    }

    /// move `mover` along one tile in provided direction.
    /// no-op if movement is impossible or provided unit has already moved this turn.
    fn try_move(&mut self, mover: u64, dir: Direction) {
        if self.moved.contains(&mover) {
            return; // already moved this turn
        }
        let Some((from, _)) = self.grid.find_unit(mover) else {
            return; // unit doesn't exist
        };
        let Some(cells_to_move) = self.chain(from, dir) else {
            return; // movement is impossible (blocked by STOP, etc.)
        };
        for (i, pos) in cells_to_move.iter().enumerate().rev() {
            let to = pos.shift(dir);
            let currently_moved = if i == 0 {
                self.grid.move_matching(*pos, to, dir, |u| u.id() == mover)
            } else {
                self.grid
                    .move_matching(*pos, to, dir, |u| self.rules.unit_has_prop(u.noun(), Property::Push))
            };
            self.moved.extend(currently_moved);
        }
    }

    /// move from `from` in direction `dir`, returning all cells that must also move this turn
    fn chain(&self, from: Pos, dir: Direction) -> Option<Vec<Pos>> {
        let mut cells_to_move = vec![from];
        let mut next = from.shift(dir);
        loop {
            if !self.grid.in_bounds(next) {
                return None;
            }
            if cell_has(self.rules, self.grid.at(next), &[Property::Stop]) {
                return None;
            }
            let pushable = self
                .grid
                .at(next)
                .units()
                .iter()
                .any(|u| !self.moved.contains(&u.id()) && self.rules.unit_has_prop(u.noun(), Property::Push));
            if !pushable {
                return Some(cells_to_move);
            }
            cells_to_move.push(next);
            next = next.shift(dir); // move to next cell
        }
    }

    fn changed(&self) -> bool {
        !self.moved.is_empty()
    }
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
        let you = self.move_you();
        let auto = self.move_autonomous();
        let select = self.move_select();
        (you || auto || select).then(|| self.reparse()); // don't reparse between movements, matches retail
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
        let dir = self.input;
        let mut movement = Movement::new(self.grid, &self.rules);
        for unit in &you {
            movement.try_move(unit.unit_id, dir);
        }
        movement.changed()
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

    fn move_autonomous(&mut self) -> bool {
        let movers = self
            .grid
            .units()
            .filter(|u| self.rules.unit_has_prop(u.noun(), Property::Move))
            .map(|u| (u.id(), u.direction()))
            .collect::<Vec<(u64, Direction)>>();

        let mut movement = Movement::new(self.grid, &self.rules);
        for (id, direction) in movers {
            movement.try_move(id, direction);
            if !movement.moved.contains(&id) {
                movement.turn(id, direction.flip());
                movement.try_move(id, direction.flip());
            }
        }
        movement.changed()
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
            self.grid
                .create_unit(new.pos, UnitKind::Object(new.into_noun), new.direction);
        }
        changed || !spawns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        eval::{Cause, Event, Turn, TurnStatus},
        unit::{Noun, UnitKind},
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

    #[track_caller]
    fn face(grid: &mut Grid, pos: Pos, noun: Noun, dir: Direction) {
        grid.at_mut(pos)
            .units_mut()
            .iter_mut()
            .find(|u| u.kind() == UnitKind::Object(noun))
            .expect("no such unit")
            .set_direction(dir);
    }

    #[track_caller]
    fn facing(grid: &Grid, noun: Noun) -> Direction {
        grid.units()
            .find(|u| u.kind() == UnitKind::Object(noun))
            .expect("no such unit")
            .direction()
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
        assert_move_result("cu ba CU IS SE", ".. ba/cu CU IS SE", Direction::East);
    }

    #[test]
    fn select_cannot_move_onto_words() {
        assert_move_result("CU IS SE cu BA", "CU IS SE cu BA", Direction::East);
    }

    #[test]
    fn select_ignores_move_rules() {
        assert_move_result(
            "CU IS SE BA IS PU cu ba ..",
            "CU IS SE BA IS PU .. ba/cu ..",
            Direction::East,
        );
        assert_move_result(
            "CU IS SE BA IS ST cu ba ..",
            "CU IS SE BA IS ST .. ba/cu ..",
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
        assert_move_result("WT IS SI wt", "WT IS SI wt", Direction::East);
    }

    #[test]
    fn test_sink_mutually() {
        assert_move_result("WT IS SI RO IS SI ro/wt", "WT IS SI RO IS SI ..", Direction::East);
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
            "BA IS YO RO IS PU WA IS DE .. ba wa/ro",
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
            "BA IS YO BA HA RO WT IS DE .. wt/ro",
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

    #[test]
    fn stacked_transform() {
        assert_move_result("RO IS BA ro/wa", "RO IS BA ba/wa", Direction::East);
    }

    #[test]
    fn stacked_push_only_pushes_pushable() {
        assert_move_result(
            "BA IS YO RO IS PU ba ro/fl ..",
            "BA IS YO RO IS PU .. fl/ba ro",
            Direction::East,
        );
    }

    #[test]
    fn created_has_unit_should_face_same_direction() {
        let mut grid = Grid::from_ascii("WT IS SI RO HA KE wt/ro");
        face(&mut grid, Pos::new(6, 0), Noun::Rock, Direction::North);
        Turn::new(&mut grid, Direction::West).run();
        assert_eq!(grid.to_ascii(), "WT IS SI RO HA KE ke");
        assert_eq!(facing(&grid, Noun::Key), Direction::North);
    }

    #[test]
    fn created_is_unit_should_face_same_direction() {
        let mut grid = Grid::from_ascii("RO IS KE ro");
        face(&mut grid, Pos::new(3, 0), Noun::Rock, Direction::North);
        Turn::new(&mut grid, Direction::West).run();
        assert_eq!(grid.to_ascii(), "RO IS KE ke");
        assert_eq!(facing(&grid, Noun::Key), Direction::North);
    }

    #[test]
    fn units_should_not_be_double_pushed() {
        assert_move_result(
            "BA IS YO RO IS PU ba ro/ba .. ..",
            "BA IS YO RO IS PU .. ba ro/ba ..",
            Direction::East,
        );
    }

    #[test]
    fn basic_move_movement() {
        assert_move_result("BA IS MO ba ..", "BA IS MO .. ba", Direction::West);
        assert_move_result("BA IS MO ba .. ba ..", "BA IS MO .. ba .. ba", Direction::West);
        assert_move_result(".. ba ..", ".. ba ..", Direction::West);
    }

    #[test]
    fn move_blocked_by_edge_should_turn() {
        assert_move_result("BA IS MO .. ba", "BA IS MO ba ..", Direction::North);
    }

    #[test]
    fn move_blocked_by_unit_should_turn() {
        assert_move_result(
            "BA IS MO RO IS ST .. ba RO",
            "BA IS MO RO IS ST ba .. RO",
            Direction::North,
        );
        assert_move_result(
            "BA IS MO RO IS PU .. ba RO",
            "BA IS MO RO IS PU ba .. RO",
            Direction::North,
        );
        assert_move_result(
            "BA IS MO RO IS PU .. ba RO RO RO",
            "BA IS MO RO IS PU ba .. RO RO RO",
            Direction::North,
        );
    }

    #[test]
    fn move_can_push_units() {
        assert_move_result(
            "BA IS MO RO IS PU ba RO ..",
            "BA IS MO RO IS PU .. ba RO",
            Direction::North,
        );
        assert_move_result(
            "BA IS MO RO IS PU ba RO RO RO RO ..",
            "BA IS MO RO IS PU .. ba RO RO RO RO",
            Direction::North,
        );
    }

    #[test]
    fn move_can_push_you_units() {
        assert_move_result(
            "RO IS MO BA IS YO BA IS PU ro ba ..",
            "RO IS MO BA IS YO BA IS PU .. ro ba",
            Direction::North,
        );
    }

    #[test]
    fn move_can_stack_onto_units() {
        assert_move_result("BA IS MO ba ro", "BA IS MO .. ro/ba", Direction::North)
    }

    #[test]
    fn move_blocked_both_directions_still_turns() {
        let mut grid = Grid::from_ascii("BA IS MO RO IS ST ro ba ro");
        face(&mut grid, Pos::new(7, 0), Noun::Baba, Direction::East);
        Turn::new(&mut grid, Direction::West).run();
        assert_eq!(grid.to_ascii(), "BA IS MO RO IS ST ro ba ro");
        assert_eq!(facing(&grid, Noun::Baba), Direction::West);
        Turn::new(&mut grid, Direction::West).run();
        assert_eq!(grid.to_ascii(), "BA IS MO RO IS ST ro ba ro");
        assert_eq!(facing(&grid, Noun::Baba), Direction::East);
    }

    #[test]
    fn move_can_push_move_units() {
        let mut grid = Grid::from_ascii("RO IS MO RO IS PU BA IS YO .. .. .. ro ba .. .. ..");
        face(&mut grid, Pos::new(12, 0), Noun::Rock, Direction::North);
        Turn::new(&mut grid, Direction::West).run();
        assert_eq!(grid.to_ascii(), "RO IS MO RO IS PU BA IS YO .. ro .. ba .. .. .. ..");
    }

    #[test]
    fn move_and_you_units_move_twice() {
        assert_move_result(
            "BA IS YO BA IS MO ba .. ..",
            "BA IS YO BA IS MO .. .. ba",
            Direction::East,
        );
    }

    #[test]
    fn move_does_not_run_on_turn_rule_created() {
        assert_move_result(
            "BA IS YO ba KE .. IS MO .. ke ..",
            "BA IS YO .. ba KE IS MO .. ke ..",
            Direction::East,
        );
    }

    #[test]
    fn move_runs_on_turn_rule_broken() {
        assert_move_result(
            "BA IS YO KE ..\n.. .. ba IS ..\nke .. .. MO ..",
            "BA IS YO KE ..\n.. .. .. ba IS\n.. ke .. MO ..",
            Direction::East,
        );
    }

    #[test]
    fn move_onto_sink() {
        let mut grid = Grid::from_ascii("RO IS MO WT IS SI ro wt");
        let result = Turn::new(&mut grid, Direction::North).run();
        assert_eq!(grid.to_ascii(), "RO IS MO WT IS SI .. ..");
        assert_eq!(result.events.len(), 2);
    }

    #[test]
    fn move_defeat_unit_onto_you() {
        let mut grid = Grid::from_ascii("RO IS MO RO IS DE BA IS YO ro ba");
        let result = Turn::new(&mut grid, Direction::North).run();
        assert_eq!(grid.to_ascii(), "RO IS MO RO IS DE BA IS YO .. ro");
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn move_win_unit_onto_you() {
        let mut grid = Grid::from_ascii("RO IS MO RO IS WI BA IS YO ro ba");
        let result = Turn::new(&mut grid, Direction::North).run();
        assert_eq!(grid.to_ascii(), "RO IS MO RO IS WI BA IS YO .. ba/ro");
        assert_eq!(result.status, TurnStatus::Win);
    }

    #[test]
    fn move_unit_onto_you_and_sink() {
        assert_move_result(
            "RO IS MO BA IS YO WT IS SI ro ba/wt",
            "RO IS MO BA IS YO WT IS SI .. ..",
            Direction::North,
        )
    }
}
