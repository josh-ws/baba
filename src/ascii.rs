use crate::{
    lex::Run,
    rule::{Complement, Rule},
    unit::{Noun, Operator, Property, Text, Unit, UnitKind},
    world::{Cell, Grid, Pos},
};

const CODES: &[(&str, UnitKind)] = &[
    ("ba", UnitKind::Object(Noun::Baba)),
    ("ke", UnitKind::Object(Noun::Keke)),
    ("BA", UnitKind::Text(Text::Noun(Noun::Baba))),
    ("KE", UnitKind::Text(Text::Noun(Noun::Keke))),
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
                return c;
            }
        }
        unreachable!("no ascii code for {self:?}");
    }
}

impl Cell {
    pub fn to_ascii(&self) -> &'static str {
        match self.units().last() {
            Some(u) => u.kind().to_ascii(),
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
                grid.at_mut(Pos::new(x as i32, y as i32))
                    .units_mut()
                    .push(Unit::new(id, UnitKind::from_ascii(code)));
            }
        }
        grid
    }

    pub fn to_ascii(&self) -> String {
        if self.width() == 0 {
            return String::new();
        }
        self.cells()
            .chunks(self.width() as usize)
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
            .map(|f| UnitKind::Text(*f))
            .map(|f| f.to_ascii().to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl Rule {
    pub fn to_ascii(&self) -> String {
        let subject = UnitKind::Text(Text::Noun(self.subject)).to_ascii();
        let operator = UnitKind::Text(Text::Operator(self.operator)).to_ascii();
        let complement = match self.complement {
            Complement::Transformation(n) => UnitKind::Text(Text::Noun(n)),
            Complement::Property(p) => UnitKind::Text(Text::Property(p)),
        };
        format!("{subject} {operator} {}", complement.to_ascii())
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
