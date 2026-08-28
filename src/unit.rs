#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Noun {
    Baba,
    Keke,
    Text,
    Wall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Is,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Property {
    You,
    Stop,
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

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn kind(&self) -> UnitKind {
        self.kind
    }

    pub fn noun(&self) -> Noun {
        match self.kind() {
            UnitKind::Object(noun) => noun,
            UnitKind::Text(_) => Noun::Text,
        }
    }
}
