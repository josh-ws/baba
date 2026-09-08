use crate::{
    lex::Run,
    rule::Rule,
    unit::{Unit, UnitKind, search_unit},
    world::{Cell, Grid, Pos},
};

impl UnitKind {
    pub fn from_ascii(s: &str) -> Self {
        search_unit(|d| d.code == s)
            .unwrap_or_else(|| panic!("unrecognised code {s}"))
            .kind
    }

    #[cfg(test)]
    pub fn to_ascii(self) -> &'static str {
        use crate::unit::lookup_unit;

        lookup_unit(self).code
    }
}

impl Cell {
    #[cfg(test)]
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

    #[cfg(test)]
    pub fn to_ascii(&self) -> String {
        if self.width() == 0 {
            return String::new();
        }
        self.cells()
            .chunks(self.width() as usize)
            .map(|row| row.iter().map(|cell| cell.to_ascii()).collect::<Vec<&str>>().join(" "))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl Run {
    #[cfg(test)]
    pub fn to_ascii(&self) -> String {
        self.slots()
            .iter()
            .map(|slot| {
                slot.iter()
                    .map(|t| UnitKind::Text(*t).to_ascii())
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Rule {
    #[cfg(test)]
    pub fn to_ascii(&self) -> String {
        use crate::{rule::Complement, unit::Text};

        let subject = UnitKind::Text(Text::Noun(self.subject)).to_ascii();
        let operator = UnitKind::Text(Text::Operator(self.operator)).to_ascii();
        let complement = match self.complement {
            Complement::Noun(n) => UnitKind::Text(Text::Noun(n)),
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
}
