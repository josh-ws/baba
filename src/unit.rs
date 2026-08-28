#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Noun {
    Baba,
    Keke,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Is,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Property {
    You,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Text {
    Noun(Noun),
    Operator(Operator),
    Property(Property),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Object(Noun),
    Text(Text),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    id: u64,
    kind: UnitKind,
}

impl Unit {
    pub fn new(id: u64, kind: UnitKind) -> Self {
        Self { id, kind }
    }

    pub fn kind(&self) -> UnitKind {
        self.kind
    }
}
