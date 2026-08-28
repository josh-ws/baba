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
        let codes: Vec<&str> = src.trim().split_whitespace().collect();
        let w = codes.len() as i32;
        let mut grid = Self::empty(w, 1);
        for (i, code) in codes.into_iter().enumerate() {
            if code == ".." {
                continue;
            }
            let id = grid.next();
            grid.cells[i]
                .units
                .push(Unit::new(id, UnitKind::from_ascii(code)));
        }
        grid
    }

    pub fn to_ascii(&self) -> String {
        self.cells
            .iter()
            .map(|cell| cell.to_ascii())
            .collect::<Vec<&str>>()
            .join(" ")
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

    #[test]
    fn ascii_round_trip() {
        let case = "BA IS YO .. ba";
        assert_eq!(Grid::from_ascii(case).to_ascii(), case);
    }

    #[test]
    fn every_code_round_trips() {
        for (c, kind) in CODES {
            assert_eq!(UnitKind::from_ascii(c), *kind);
            assert_eq!(kind.to_ascii(), *c);
        }
    }
}
