use crate::{
    Grid, Text,
    world::{Direction, Pos},
};

const MIN_RULE_LENGTH: usize = 3;

#[derive(Debug)]
pub struct Run {
    words: Vec<Text>,
}

impl Run {
    pub fn new() -> Self {
        Self { words: Vec::new() }
    }

    pub fn words(&self) -> &Vec<Text> {
        &self.words
    }
}

fn scan_line(grid: &Grid, start: Pos, dir: Direction) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut current_run = Run::new();
    for pos in grid.iter(start, dir) {
        match grid.at(pos).first_word() {
            Some(word) => current_run.words.push(word),
            None => {
                if !current_run.words.is_empty() {
                    runs.push(current_run);
                    current_run = Run::new();
                }
            }
        }
    }
    if !current_run.words.is_empty() {
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
    runs.retain(|f| f.words.len() >= MIN_RULE_LENGTH);
    runs
}

#[cfg(test)]
mod tests {
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
    }
}
