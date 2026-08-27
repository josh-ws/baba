use crate::{Cell, Grid, Noun, Operator, Property, Text, Unit, UnitKind};

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
        match self {
            UnitKind::Object(Noun::Baba) => "ba",
            UnitKind::Text(Text::Noun(Noun::Baba)) => "BA",
            UnitKind::Text(Text::Operator(Operator::Is)) => "IS",
            UnitKind::Text(Text::Property(Property::You)) => "YO",
        }
    }
}

impl Cell {
    pub fn to_ascii(&self) -> String {
        if self.units.is_empty() {
            "..".to_string()
        } else {
            self.units
                .first()
                .unwrap()
                .clone()
                .kind
                .to_ascii()
                .to_string()
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
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn ascii_round_trip() {
        let case = "BA IS YO .. ba";
        assert_eq!(Grid::from_ascii(case).to_ascii(), case);
    }
}
