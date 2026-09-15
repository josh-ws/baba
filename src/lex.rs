use crate::{
    unit::Text,
    world::{Direction, Grid, Pos},
};

const MIN_RULE_LENGTH: usize = 3;
const MAX_READINGS: usize = 3000;

pub type Slot = Vec<Text>;

#[derive(Debug, Default)]
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn slot(&self, i: usize) -> Option<&[Text]> {
        self.slots.get(i).map(Vec::as_slice)
    }

    pub fn slots(&self) -> &Vec<Slot> {
        &self.slots
    }

    pub fn variants(&self) -> Vec<Vec<Text>> {
        let count = self
            .slots()
            .iter()
            .fold(1 as usize, |n, slot| n.saturating_mul(slot.len()));
        if count > MAX_READINGS {
            return Vec::new();
        }
        let mut lines = vec![vec![]];
        for slot in &self.slots {
            let mut next = Vec::new();
            for line in &lines {
                for word in slot {
                    next.push([line.as_slice(), &[*word]].concat());
                }
            }
            lines = next;
        }
        lines
    }
}

fn scan_line(grid: &Grid, start: Pos, dir: Direction) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut current_run = Run::new();
    for pos in grid.iter(start, dir) {
        let words = grid.at(pos).words();
        if words.is_empty() {
            if !current_run.is_empty() {
                runs.push(current_run);
                current_run = Run::new();
            }
        } else {
            current_run.slots.push(words);
        }
    }
    if !current_run.is_empty() {
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
#[rustfmt::skip]
mod tests {
    use crate::{
        unit::{Noun, Operator, Property},
        world::Grid,
    };

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

    #[test]
    fn lex_stacked_text() {
        assert_lex_match("BA/RO IS/HA YO/RO", vec!["BA/RO IS/HA YO/RO"]);
    }

    #[test]
    fn lex_stacked_text_dedupes() {
        assert_lex_match("BA/BA IS YO", vec!["BA IS YO"]);
    }

    #[test]
    fn variants() {
        let run = Run {
            slots: vec![
                vec![Text::Noun(Noun::Baba), Text::Noun(Noun::Key)],
                vec![Text::Operator(Operator::Is)],
                vec![Text::Property(Property::You), Text::Property(Property::Push)],
            ],
        };
        let variants = run.variants();
        assert_eq!(4, variants.len());
        assert_eq!("[Noun(Baba), Operator(Is), Property(You)]",format!("{:?}", variants[0]));
        assert_eq!("[Noun(Baba), Operator(Is), Property(Push)]",format!("{:?}", variants[1]));
        assert_eq!("[Noun(Key), Operator(Is), Property(You)]", format!("{:?}", variants[2]));
        assert_eq!("[Noun(Key), Operator(Is), Property(Push)]", format!("{:?}", variants[3]));
    }
}
