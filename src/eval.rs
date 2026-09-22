use std::collections::HashSet;

use crate::{
    lex::lex,
    parse::parse,
    query::{
        UnitRef, any_cell_has, cell_has, query_defeat, query_has, query_is_noun, query_is_property, query_melt,
        query_selected, query_sink,
    },
    rule::Rules,
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
    Melt,
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
            self.grid.turn_unit(mover, dir); // movement is impossible (blocked by STOP, etc.) but still turn
            return;
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
        let movers = self.move_autonomous(Property::Move, true);
        let auto = self.move_autonomous(Property::Auto, false);
        let select = self.move_select();
        (you || movers || auto || select).then(|| self.reparse()); // don't reparse between movements, matches retail
        self.handle_transforms().then(|| self.reparse());
        self.handle_sink().then(|| self.reparse());
        self.handle_defeats().then(|| self.reparse());
        self.handle_melt().then(|| self.reparse());
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

    fn move_autonomous(&mut self, filter: Property, can_flip: bool) -> bool {
        let movers = self
            .grid
            .units()
            .filter_map(|u| {
                if self.rules.unit_has_prop(u.noun(), filter) {
                    Some((u.id(), u.direction()))
                } else {
                    None
                }
            })
            .collect::<Vec<(u64, Direction)>>();

        let mut movement = Movement::new(self.grid, &self.rules);
        for (id, direction) in movers {
            movement.try_move(id, direction);
            if can_flip && !movement.moved.contains(&id) {
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

    fn handle_melt(&mut self) -> bool {
        let melted = query_melt(&self.rules, self.grid);
        self.destroy_and_create(&melted, Cause::Melt)
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
#[rustfmt::skip]
mod tests {
    use crate::{
        eval::{Cause, Event, Turn, TurnResult, TurnStatus}, unit::{Noun, UnitKind}, world::{Direction, Grid, Pos},
    };

    /// play every key of `input` (`>`, `<`, `^`, `v`) against `grid`, returning the last turn's result
    #[track_caller]
    fn play(grid: &mut Grid, input: &str) -> TurnResult {
        let mut result = None;
        for key in input.chars() {
            let direction = match key {
                '>' => Direction::East,
                '<' => Direction::West,
                '^' => Direction::North,
                'v' => Direction::South,
                _ => panic!("invalid key {key}"),
            };
            result = Some(Turn::new(grid, direction).run());
        }
        result.expect("no input given")
    }

    #[track_caller]
    fn expect(setup: &str, input: &str, exp: &str) {
        let mut grid = Grid::from_ascii(setup);
        play(&mut grid, input);
        assert_eq!(grid.to_ascii(), exp);
    }

    #[track_caller]
    fn expect_status(setup: &str, input: &str, status: TurnStatus) {
        let mut grid = Grid::from_ascii(setup);
        assert_eq!(play(&mut grid, input).status, status);
    }

    #[track_caller]
    fn expect_events(setup: &str, input: &str, events: &[Event]) {
        let mut grid = Grid::from_ascii(setup);
        assert_eq!(play(&mut grid, input).events, events);
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

    #[track_caller]
    fn expect_facing(setup: &str, input: &str, exp: &str, noun: Noun, dir: Direction) {
        let mut grid = Grid::from_ascii(setup);
        play(&mut grid, input);
        assert_eq!(grid.to_ascii(), exp);
        assert_eq!(facing(&grid, noun), dir);
    }


    #[test]
    fn you() {
        // basic movement
        expect("BA IS YO ba ..", ">", "BA IS YO .. ba");
        expect("BA IS YO .. ba", "<", "BA IS YO ba ..");
        expect("BA IS YO ba \n.. .. .. ..", "v", "BA IS YO ..\n.. .. .. ba");
        expect("BA IS YO .. \n.. .. .. ba", "^", "BA IS YO ba\n.. .. .. ..");
        // multi move
        expect("BA IS YO ba ba ba ..", ">", "BA IS YO .. ba ba ba");
        // blocked on edge
        expect("BA IS YO .. ba", ">", "BA IS YO .. ba");
        expect("ba .. BA IS YO", "<", "ba .. BA IS YO");
        expect("BA IS YO .. ba", "^", "BA IS YO .. ba");
        expect("BA IS YO .. ba", "v", "BA IS YO .. ba");
        // blocked by unit
        expect("BA IS YO .. ba wa .. WA IS ST", ">", "BA IS YO .. ba wa .. WA IS ST");
    }

    #[test]
    fn blocked_turns() {
        // units start facing east, so every blocked case here tries some other direction
        // control: a successful move faces the direction moved
        expect_facing("BA IS YO ba\n.. .. .. ..", "v", "BA IS YO ..\n.. .. .. ba", Noun::Baba, Direction::South);
        expect_facing("BA IS YO ..\n.. .. .. ba", "^", "BA IS YO ba\n.. .. .. ..", Noun::Baba, Direction::North);
        // blocked by the edge: turns, doesn't move
        expect_facing("BA IS YO .. ba", "^", "BA IS YO .. ba", Noun::Baba, Direction::North);
        expect_facing("BA IS YO .. ba", "v", "BA IS YO .. ba", Noun::Baba, Direction::South);
        expect_facing("ba .. BA IS YO", "<", "ba .. BA IS YO", Noun::Baba, Direction::West);
        // blocked by `stop`
        expect_facing(
            "BA IS YO WA IS ST ba\n.. .. .. .. .. .. wa",
            "v",
            "BA IS YO WA IS ST ba\n.. .. .. .. .. .. wa",
            Noun::Baba,
            Direction::South,
        );
        // blocked by level, which is inherently `stop`
        expect_facing(
            "BA IS YO\nle .. ..\nba .. ..",
            "^",
            "BA IS YO\nle .. ..\nba .. ..",
            Noun::Baba,
            Direction::North,
        );
        // blocked by a `push` chain that is itself blocked
        expect_facing(
            "BA IS YO RO IS PU ba\n.. .. .. .. .. .. ro",
            "v",
            "BA IS YO RO IS PU ba\n.. .. .. .. .. .. ro",
            Noun::Baba,
            Direction::South,
        );
        // turning back east from another facing, so the default can't satisfy it
        let mut grid = Grid::from_ascii("BA IS YO .. ba");
        face(&mut grid, Pos::new(4, 0), Noun::Baba, Direction::North);
        play(&mut grid, ">");
        assert_eq!(grid.to_ascii(), "BA IS YO .. ba");
        assert_eq!(facing(&grid, Noun::Baba), Direction::East);
    }

    #[test]
    fn push() {
        // basic push
        expect("BA IS YO RO IS PU .. ba ro ..", ">", "BA IS YO RO IS PU .. .. ba ro");
        expect("BA IS YO RO IS PU .. ro ba ..", "<", "BA IS YO RO IS PU ro ba .. ..");
        expect("BA IS YO ..\nRO IS PU ro\n.. .. .. ba", "^", "BA IS YO ro\nRO IS PU ba\n.. .. .. ..");
        expect("BA IS YO ba\nRO IS PU ro\n.. .. .. ..", "v", "BA IS YO ..\nRO IS PU ba\n.. .. .. ro");
        // chaining
        expect("BA IS YO RO IS PU ba ro ro .. ro ..", ">", "BA IS YO RO IS PU .. ba ro ro ro ..");
        // you + push
        expect("BA IS YO BA IS PU ba ba .. ..", ">", "BA IS YO BA IS PU .. ba ba ..");
        // push blocked by `stop`
        expect("BA IS YO RO IS PU WA IS ST ba ro wa ..", ">", "BA IS YO RO IS PU WA IS ST ba ro wa ..");
        // text is pushable
        expect("BA IS YO ba BA ..", ">", "BA IS YO .. ba BA");
        // only the pushable half of a stack is pushed
        expect("BA IS YO RO IS PU ba ro/fl ..", ">", "BA IS YO RO IS PU .. fl/ba ro");
        // a unit that already moved this phase is not pushed a second time
        expect("BA IS YO RO IS PU ba ro/ba .. ..", ">", "BA IS YO RO IS PU .. ba ro/ba ..");
    }

    #[test]
    fn win() {
        expect_status("BA IS YO RO IS WI ba ro", ">", TurnStatus::Win);
        expect_status("BA IS YO BA IS WI ba", "^", TurnStatus::Win);
        expect_status("BA IS YO ba BA .. IS WI", ">", TurnStatus::Win);
    }

    #[test]
    fn transform() {
        expect("BA IS RO ba", ">", "BA IS RO ro");
        expect("BA IS RO IS BA ba ro", ">", "BA IS RO IS BA ro ba");
        // no loopback: the fresh rock is not transformed back this turn
        expect("BA IS RO IS BA ba", ">", "BA IS RO IS BA ro");
        // `X IS X` cancels any other transform of X
        expect("BA IS RO BA IS BA ba", ">", "BA IS RO BA IS BA ba");
        // only the named unit of a stack transforms
        expect("RO IS BA ro/wa", ">", "RO IS BA ba/wa");
        // the created unit inherits the old one's facing
        let mut grid = Grid::from_ascii("RO IS KE ro");
        face(&mut grid, Pos::new(3, 0), Noun::Rock, Direction::North);
        play(&mut grid, "<");
        assert_eq!(grid.to_ascii(), "RO IS KE ke");
        assert_eq!(facing(&grid, Noun::Key), Direction::North);
    }

    #[test]
    fn level() {
        // level is inherently `stop`, and that cannot be overridden
        expect("BA IS YO ba le", ">", "BA IS YO ba le");
        expect("BA IS YO LE IS PU ba le ..", ">", "BA IS YO LE IS PU ba le ..");
    }

    #[test]
    fn select() {
        // moves onto objects
        expect("cu ba CU IS SE", ">", ".. ba/cu CU IS SE");
        // but never onto text
        expect("CU IS SE cu BA", ">", "CU IS SE cu BA");
        // and ignores `push` and `stop`
        expect("CU IS SE BA IS PU cu ba ..", ">", "CU IS SE BA IS PU .. ba/cu ..");
        expect("CU IS SE BA IS ST cu ba ..", ">", "CU IS SE BA IS ST .. ba/cu ..");
    }

    #[test]
    fn sink() {
        // destroys whatever it shares a cell with, and itself
        expect("WA IS SI BA IS YO ba wa", ">", "WA IS SI BA IS YO .. ..");
        expect("WA IS SI WA IS YO ba wa", "<", "WA IS SI WA IS YO .. ..");
        // alone on a cell it survives
        expect("WT IS SI wt", ">", "WT IS SI wt");
        // two sinks sink each other
        expect("WT IS SI RO IS SI ro/wt", ">", "WT IS SI RO IS SI ..");
        // sink resolves before the win check
        expect_status("WT IS SI WT IS WI BA IS YO ba wt", ">", TurnStatus::Continue);
        expect("WT IS SI WT IS WI BA IS YO ba wt", ">", "WT IS SI WT IS WI BA IS YO .. ..");
        // one event per destroyed unit
        expect_events("BA IS YO WT IS SI ba wt", ">", &[
            Event::Destroyed { pos: Pos::new(7, 0), cause: Cause::Sink },
            Event::Destroyed { pos: Pos::new(7, 0), cause: Cause::Sink },
        ]);
    }

    #[test]
    fn defeat() {
        expect("BA IS YO RO IS DE ba ro", ">", "BA IS YO RO IS DE .. ro");
        // you can defeat itself
        expect("BA IS YO BA IS DE ba ..", ">", "BA IS YO BA IS DE .. ..");
        // defeat resolves before the win check
        expect_status("BA IS YO RO IS DE RO IS WI ba ro", ">", TurnStatus::Continue);
        // only the `you` unit dies, anything else sharing the cell stays
        expect("BA IS YO RO IS PU WA IS DE ba ro wa", ">", "BA IS YO RO IS PU WA IS DE .. ba wa/ro");
        // one event per destroyed unit
        expect_events("BA IS YO WT IS DE ba wt", ">", &[
            Event::Destroyed { pos: Pos::new(7, 0), cause: Cause::Defeat },
        ]);
    }

    #[test]
    fn has() {
        // a destroyed unit leaves its `has` unit behind, however it died
        expect("BA IS YO BA HA RO WT IS SI ba wt", ">", "BA IS YO BA HA RO WT IS SI .. ro");
        expect("BA IS YO BA HA RO WT IS DE ba wt", ">", "BA IS YO BA HA RO WT IS DE .. wt/ro");
        // the created unit inherits the destroyed one's facing
        let mut grid = Grid::from_ascii("WT IS SI RO HA KE wt/ro");
        face(&mut grid, Pos::new(6, 0), Noun::Rock, Direction::North);
        play(&mut grid, "<");
        assert_eq!(grid.to_ascii(), "WT IS SI RO HA KE ke");
        assert_eq!(facing(&grid, Noun::Key), Direction::North);
    }

    #[test]
    fn r#move() {
        // travels along its own facing, not the input
        expect("BA IS MO ba ..", "<", "BA IS MO .. ba");
        expect("BA IS MO ba .. ba ..", "<", "BA IS MO .. ba .. ba");
        expect(".. ba ..", "<", ".. ba ..");
        // blocked by the edge: turn around
        expect("BA IS MO .. ba", "^", "BA IS MO ba ..");
        // blocked by a unit: turn around
        expect("BA IS MO RO IS ST .. ba RO", "^", "BA IS MO RO IS ST ba .. RO");
        expect("BA IS MO RO IS PU .. ba RO", "^", "BA IS MO RO IS PU ba .. RO");
        expect("BA IS MO RO IS PU .. ba RO RO RO", "^", "BA IS MO RO IS PU ba .. RO RO RO");
        // pushes what it walks into
        expect("BA IS MO RO IS PU ba RO ..", "^", "BA IS MO RO IS PU .. ba RO");
        expect("BA IS MO RO IS PU ba RO RO RO RO ..", "^", "BA IS MO RO IS PU .. ba RO RO RO RO");
        expect("RO IS MO BA IS YO BA IS PU ro ba ..", "^", "RO IS MO BA IS YO BA IS PU .. ro ba");
        // stacks onto whatever isn't `stop`
        expect("BA IS MO ba ro", "^", "BA IS MO .. ro/ba");
        // a unit that is both `you` and `move` moves in both phases
        expect("BA IS YO BA IS MO ba .. ..", ">", "BA IS YO BA IS MO .. .. ba");
        // the phase runs on the rules as they were before the turn's movement
        expect("BA IS YO ba KE .. IS MO .. ke ..", ">", "BA IS YO .. ba KE IS MO .. ke ..");
        expect("BA IS YO KE ..\n.. .. ba IS ..\nke .. .. MO ..", ">", "BA IS YO KE ..\n.. .. .. ba IS\n.. ke .. MO ..");
        // blocked both ways: no movement, but still turns each turn
        let mut grid = Grid::from_ascii("BA IS MO RO IS ST ro ba ro");
        face(&mut grid, Pos::new(7, 0), Noun::Baba, Direction::East);
        play(&mut grid, "<");
        assert_eq!(grid.to_ascii(), "BA IS MO RO IS ST ro ba ro");
        assert_eq!(facing(&grid, Noun::Baba), Direction::West);
        play(&mut grid, "<");
        assert_eq!(grid.to_ascii(), "BA IS MO RO IS ST ro ba ro");
        assert_eq!(facing(&grid, Noun::Baba), Direction::East);
        // a `move` unit can push another `move` unit
        let mut grid = Grid::from_ascii("RO IS MO RO IS PU BA IS YO .. .. .. ro ba .. .. ..");
        face(&mut grid, Pos::new(12, 0), Noun::Rock, Direction::North);
        play(&mut grid, "<");
        assert_eq!(grid.to_ascii(), "RO IS MO RO IS PU BA IS YO .. ro .. ba .. .. .. ..");
        // moving into sink/defeat/win resolves them as if `you` had moved
        let mut grid = Grid::from_ascii("RO IS MO WT IS SI ro wt");
        let result = play(&mut grid, "^");
        assert_eq!(grid.to_ascii(), "RO IS MO WT IS SI .. ..");
        assert_eq!(result.events.len(), 2);
        let mut grid = Grid::from_ascii("RO IS MO RO IS DE BA IS YO ro ba");
        let result = play(&mut grid, "^");
        assert_eq!(grid.to_ascii(), "RO IS MO RO IS DE BA IS YO .. ro");
        assert_eq!(result.events.len(), 1);
        expect_status("RO IS MO RO IS WI BA IS YO ro ba", "^", TurnStatus::Win);
        expect("RO IS MO RO IS WI BA IS YO ro ba", "^", "RO IS MO RO IS WI BA IS YO .. ba/ro");
        expect("RO IS MO BA IS YO WT IS SI ro ba/wt", "^", "RO IS MO BA IS YO WT IS SI .. ..");
    }

    #[test]
    fn auto() {
        // like `move`, but never turns around
        expect("BA IS AU ba ..", "<", "BA IS AU .. ba");
        expect("BA IS AU ba .. ba ..", "<", "BA IS AU .. ba .. ba");
        expect(".. ba ..", "<", ".. ba ..");
        // blocked by the edge: stay put, keep facing
        expect("BA IS AU .. ba", "^", "BA IS AU .. ba");
        // blocked by a unit: stay put, keep facing
        expect("BA IS AU RO IS ST .. ba RO", "^", "BA IS AU RO IS ST .. ba RO");
        expect("BA IS AU RO IS PU .. ba RO", "^", "BA IS AU RO IS PU .. ba RO");
        expect("BA IS AU RO IS PU .. ba RO RO RO", "^", "BA IS AU RO IS PU .. ba RO RO RO");
        // pushes what it walks into
        expect("BA IS AU RO IS PU ba RO ..", "^", "BA IS AU RO IS PU .. ba RO");
        expect("BA IS AU RO IS PU ba RO RO RO RO ..", "^", "BA IS AU RO IS PU .. ba RO RO RO RO");
        expect("RO IS AU BA IS YO BA IS PU ro ba ..", "^", "RO IS AU BA IS YO BA IS PU .. ro ba");
        // stacks onto whatever isn't `stop`
        expect("BA IS AU ba ro", "^", "BA IS AU .. ro/ba");
        // each movement phase gets its own step
        expect("BA IS YO BA IS AU ba .. ..", ">", "BA IS YO BA IS AU .. .. ba");
        expect("BA IS YO BA IS AU BA IS MO ba .. .. ..", ">", "BA IS YO BA IS AU BA IS MO .. .. .. ba");
        // the phase runs on the rules as they were before the turn's movement
        expect("BA IS YO ba KE .. IS AU .. ke ..", ">", "BA IS YO .. ba KE IS AU .. ke ..");
        expect("BA IS YO KE ..\n.. .. ba IS ..\nke .. .. AU ..", ">", "BA IS YO KE ..\n.. .. .. ba IS\n.. ke .. AU ..");
        // an `auto` unit can push a `move` unit
        let mut grid = Grid::from_ascii("RO IS AU RO IS PU BA IS YO .. .. .. ro ba .. .. ..");
        face(&mut grid, Pos::new(12, 0), Noun::Rock, Direction::North);
        play(&mut grid, "<");
        assert_eq!(grid.to_ascii(), "RO IS AU RO IS PU BA IS YO .. ro .. ba .. .. .. ..");
        // moving into sink/defeat/win resolves them as if `you` had moved
        let mut grid = Grid::from_ascii("RO IS AU WT IS SI ro wt");
        let result = play(&mut grid, "^");
        assert_eq!(grid.to_ascii(), "RO IS AU WT IS SI .. ..");
        assert_eq!(result.events.len(), 2);
        let mut grid = Grid::from_ascii("RO IS AU RO IS DE BA IS YO ro ba");
        let result = play(&mut grid, "^");
        assert_eq!(grid.to_ascii(), "RO IS AU RO IS DE BA IS YO .. ro");
        assert_eq!(result.events.len(), 1);
        expect_status("RO IS AU RO IS WI BA IS YO ro ba", "^", TurnStatus::Win);
        expect("RO IS AU RO IS WI BA IS YO ro ba", "^", "RO IS AU RO IS WI BA IS YO .. ba/ro");
        expect("RO IS AU BA IS YO WT IS SI ro ba/wt", "^", "RO IS AU BA IS YO WT IS SI .. ..");
    }

    #[test]
    #[ignore = "needs a rewrite of movement"]
    fn you_moves_before_stop() {
        expect("BA AN FL IS YO AN ST ba fl .. ..", ">", "BA AN FL IS YO AN ST .. ba fl ..");
    }
}
