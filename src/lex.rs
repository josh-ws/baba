use crate::{
    unit::Text,
    world::{Direction, Grid, Pos},
};

const MIN_RULE_LENGTH: usize = 3;

pub type Slot = Vec<Text>;

#[derive(Debug)]
pub struct Run {
    slots: Vec<Slot>,
}

impl Run {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn slot(&self, i: usize) -> Option<&[Text]> {
        self.slots.get(i).map(Vec::as_slice)
    }

    pub fn slots(&self) -> &Vec<Slot> {
        &self.slots
    }
}

fn scan_line(grid: &Grid, start: Pos, dir: Direction) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut current_run = Run::new();
    for pos in grid.iter(start, dir) {
        let words = grid.at(pos).words();
        if words.is_empty() {
            if current_run.len() > 0 {
                runs.push(current_run);
                current_run = Run::new();
            }
        } else {
            current_run.slots.push(words);
        }
    }
    if current_run.len() > 0 {
        runs.push(current_run);
    }
    runs
}

pub fn lex(grid: &Grid) -> Vec<Run> {
    let mut runs = Vec::new();
    for y in 0..grid.height() {
        runs.extend(scan_line(grid, Pos::new(0, y), Direction::East));
    }
    for x in 0..grid.width() {
        runs.extend(scan_line(grid, Pos::new(x, 0), Direction::South));
    }
    runs.retain(|f| f.slots.len() >= MIN_RULE_LENGTH);
    runs
}

#[cfg(test)]
mod tests {
    use crate::world::Grid;

    use super::*;

    #[track_caller]
    fn assert_lex_match(src: &str, exp: Vec<&str>) {
        let grid = Grid::from_ascii(src);
        let tokens = lex(&grid);
        assert_eq!(tokens.len(), exp.len());
        for i in 0..exp.len() {
            assert_eq!(tokens[i].to_ascii(), exp[i]);
        }
    }

    #[test]
    fn lex_single_row() {
        assert_lex_match("", vec![]);
        assert_lex_match(".. .. ..", vec![]);
        assert_lex_match("BA", vec![]);
        assert_lex_match("BA IS", vec![]);
        assert_lex_match("BA IS YO", vec!["BA IS YO"]);
        assert_lex_match("ba ba ba", vec![]);
        assert_lex_match("ba BA IS YO ba", vec!["BA IS YO"]);
        assert_lex_match("BA IS YO IS YO", vec!["BA IS YO IS YO"]);
        assert_lex_match("IS IS IS", vec!["IS IS IS"]);
        assert_lex_match("YO BA IS", vec!["YO BA IS"]);
        assert_lex_match(".. BA IS YO", vec!["BA IS YO"]);
        assert_lex_match("BA IS YO ..", vec!["BA IS YO"]);
        assert_lex_match("BA IS YO .. .. .. BA IS YO", vec!["BA IS YO", "BA IS YO"]);
        assert_lex_match("BA IS YO .. BA IS YO", vec!["BA IS YO", "BA IS YO"]);
        assert_lex_match("BA IS YO ba BA IS YO", vec!["BA IS YO", "BA IS YO"]);
        assert_lex_match("BA HA RO", vec!["BA HA RO"]);
    }
}
