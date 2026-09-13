use crate::world::Direction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Atlas {
    Sprites,
    Words,
    Tiled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Directional,
    Fixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitGroup {
    Liquid,
    Ground,
    Clutter,
    Wall,
    Item,
    Creatures,
    Characters,
    Expressive,
    Text,
    Ui,
}

// Sprite data for a unit
pub struct Sprite {
    pub atlas: Atlas,
    pub row: usize,
    pub facing: Facing,
}

// Static unit type data
pub struct UnitTypeData {
    pub kind: UnitKind,
    pub code: &'static str,
    pub sprite: Sprite,
    pub group: UnitGroup,
}

const fn word(text: Text, code: &'static str, row: usize) -> UnitTypeData {
    UnitTypeData {
        kind: UnitKind::Text(text),
        code,
        sprite: Sprite {
            atlas: Atlas::Words,
            facing: Facing::Fixed,
            row,
        },
        group: UnitGroup::Text,
    }
}

const fn object(noun: Noun, code: &'static str, row: usize, facing: Facing, group: UnitGroup) -> UnitTypeData {
    UnitTypeData {
        kind: UnitKind::Object(noun),
        code,
        sprite: Sprite {
            atlas: Atlas::Sprites,
            facing,
            row,
        },
        group,
    }
}

const fn tiled(noun: Noun, code: &'static str, row: usize, group: UnitGroup) -> UnitTypeData {
    UnitTypeData {
        kind: UnitKind::Object(noun),
        code,
        sprite: Sprite {
            atlas: Atlas::Tiled,
            facing: Facing::Fixed,
            row,
        },
        group,
    }
}

const UNIT_TYPES: &[UnitTypeData] = &[
    word(Text::Noun(Noun::Baba), "BA", 0),
    word(Text::Noun(Noun::Flag), "FL", 1),
    word(Text::Noun(Noun::Rock), "RO", 2),
    word(Text::Property(Property::You), "YO", 3),
    word(Text::Property(Property::Stop), "ST", 4),
    word(Text::Property(Property::Push), "PU", 5),
    word(Text::Property(Property::Win), "WI", 6),
    word(Text::Operator(Operator::Is), "IS", 7),
    word(Text::Noun(Noun::Wall), "WA", 8),
    word(Text::Noun(Noun::Cursor), "CU", 9),
    word(Text::Property(Property::Select), "SE", 10),
    word(Text::Noun(Noun::Level), "LE", 11),
    word(Text::Property(Property::Sink), "SI", 12),
    word(Text::Noun(Noun::Water), "WT", 13),
    word(Text::Property(Property::Defeat), "DE", 14),
    word(Text::Operator(Operator::Has), "HA", 15),
    word(Text::Noun(Noun::Key), "KE", 16),
    word(Text::Property(Property::Move), "MO", 17),
    word(Text::Property(Property::Auto), "AU", 18),
    word(Text::Noun(Noun::Lava), "LA", 19),
    object(Noun::Baba, "ba", 0, Facing::Directional, UnitGroup::Characters),
    object(Noun::Flag, "fl", 1, Facing::Fixed, UnitGroup::Item),
    object(Noun::Rock, "ro", 2, Facing::Fixed, UnitGroup::Item),
    object(Noun::Wall, "wa", 3, Facing::Fixed, UnitGroup::Wall),
    object(Noun::Cursor, "cu", 4, Facing::Fixed, UnitGroup::Text),
    object(Noun::Level, "le", 5, Facing::Fixed, UnitGroup::Ground),
    object(Noun::Path, "pa", 6, Facing::Fixed, UnitGroup::Ground),
    object(Noun::Key, "ke", 8, Facing::Fixed, UnitGroup::Item),
    tiled(Noun::Water, "wt", 1, UnitGroup::Liquid),
    tiled(Noun::Lava, "la", 4, UnitGroup::Liquid),
];

pub fn lookup_unit(kind: UnitKind) -> &'static UnitTypeData {
    UNIT_TYPES.iter().find(|p| p.kind == kind).expect("missing unit kind")
}

pub fn search_unit(pred: impl Fn(&UnitTypeData) -> bool) -> Option<&'static UnitTypeData> {
    UNIT_TYPES.iter().find(|d| pred(d))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Noun {
    Baba,
    Text,
    Wall,
    Rock,
    Flag,
    Cursor,
    Level,
    Path,
    Water,
    Key,
    Lava,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Is,
    Has,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Property {
    You,
    Stop,
    Push,
    Win,
    Select,
    Sink,
    Defeat,
    Move,
    Auto,
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
    direction: Direction,
}

impl Unit {
    pub fn new(id: u64, kind: UnitKind, dir: Direction) -> Self {
        Self {
            id,
            kind,
            direction: dir,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn kind(&self) -> UnitKind {
        self.kind
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, dir: Direction) {
        self.direction = dir;
    }

    pub fn noun(&self) -> Noun {
        match self.kind() {
            UnitKind::Object(noun) => noun,
            UnitKind::Text(_) => Noun::Text,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self.kind(), UnitKind::Object(_))
    }
}
