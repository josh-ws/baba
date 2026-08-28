use crate::{Cell, Grid, Noun, Operator, Property, Text, Unit, UnitKind, lex::Run};

const CODES: &[(&str, UnitKind)] = &[
    ("ba", UnitKind::Object(Noun::Baba)),
    ("BA", UnitKind::Text(Text::Noun(Noun::Baba))),
    ("IS", UnitKind::Text(Text::Operator(Operator::Is))),
    ("YO", UnitKind::Text(Text::Property(Property::You))),
];

impl UnitKind {
    pub fn from_ascii(s: &str) -> Self {
        for (c, kind) in CODES {
            if *c == s {
                return *kind;
            }
        }
        panic!("unrecognized code {s}");
    }

    pub fn to_ascii(self) -> &'static str {
        for (c, kind) in CODES {
            if *kind == self {
                return *c;
            }
        }
        unreachable!("no ascii code for {self:?}");
    }
}

impl Cell {
    pub fn to_ascii(&self) -> &'static str {
        match self.units.last() {
            Some(u) => u.kind.to_ascii(),
            None => "..",
        }
    }
}

impl Grid {
    pub fn from_ascii(src: &str) -> Self {
        let rows = src
            .trim()
            .lines()
            .map(|line| line.split_whitespace().collect())
            .collect::<Vec<Vec<&str>>>();
        let h = rows.len() as i32;
        let w = rows.first().map_or(0, |r| r.len()) as i32;
        let mut grid = Self::empty(w, h);
        for (y, row) in rows.iter().enumerate() {
            for (x, code) in row.iter().enumerate() {
                if *code == ".." {
                    continue;
                }
                let id = grid.next();
                grid.cells[y * w as usize + x]
                    .units
                    .push(Unit::new(id, UnitKind::from_ascii(code)));
            }
        }
        grid
    }

    pub fn to_ascii(&self) -> String {
        if self.w == 0 {
            return String::new();
        }
        self.cells
            .chunks(self.w as usize)
            .map(|row| {
                row.iter()
                    .map(|cell| cell.to_ascii())
                    .collect::<Vec<&str>>()
                    .join(" ")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl Run {
    pub fn to_ascii(&self) -> String {
        self.words()
            .iter()
            .map(|f| UnitKind::Text(f.clone()))
            .map(|f| f.to_ascii().to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_round_trip(src: &str) {
        let r = Grid::from_ascii(src).to_ascii();
        assert_eq!(r, src);
    }

    #[test]
    fn ascii_round_trip() {
        assert_round_trip("BA IS YO .. ba");
        assert_round_trip(".. BA IS YO ..\nBA IS YO .. ..\n.. .. BA IS YO");
    }

    #[test]
    fn every_code_round_trips() {
        for (c, kind) in CODES {
            assert_eq!(UnitKind::from_ascii(c), *kind);
            assert_eq!(kind.to_ascii(), *c);
        }
    }
}
