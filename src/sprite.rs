use crate::{
    unit::{Atlas, Facing, Noun, Unit, lookup_unit},
    world::{Direction, Grid, Pos},
};

pub struct SpriteFrame {
    pub atlas: Atlas,
    pub column: usize,
    pub row: usize,
    pub flip_x: bool,
}

pub fn sprite_frame(grid: &Grid, unit: &Unit, pos: Pos, wobble: usize) -> SpriteFrame {
    let data = lookup_unit(unit.kind());
    let (column, flip_x) = sprite_column(grid, unit, pos, wobble);
    SpriteFrame {
        atlas: data.sprite.atlas,
        column,
        row: sprite_row(unit, wobble),
        flip_x,
    }
}

fn direction_index(direction: Direction) -> usize {
    match direction {
        Direction::East | Direction::West => 0,
        Direction::South => 1,
        Direction::North => 2,
    }
}

// TODO(jw) PERF: Should this be a cached property on the unit?
fn tiled_index(grid: &Grid, pos: Pos, noun: Noun) -> usize {
    const DIRS: [Direction; 4] = [Direction::East, Direction::North, Direction::West, Direction::South];
    let mut index = 0;
    for (i, dir) in DIRS.iter().enumerate() {
        let shift = pos.shift(*dir);
        let joins = || {
            grid.at(shift)
                .units()
                .iter()
                .any(|u| u.noun() == noun || u.noun() == Noun::Level)
        };
        if !grid.in_bounds(shift) || joins() {
            index |= 1 << i;
        }
    }
    index
}

/// Returns column from the spritesheet for this sprite, and whether or not it should be drawn flipped
fn sprite_column(grid: &Grid, unit: &Unit, pos: Pos, wobble: usize) -> (usize, bool) {
    let data = lookup_unit(unit.kind());
    match data.sprite.atlas {
        Atlas::Words => (wobble, false),
        Atlas::Tiled => (tiled_index(grid, pos, unit.noun()), false),
        Atlas::Sprites => {
            let (dir, flip) = sprite_facing(unit.direction(), data.sprite.facing);
            (wobble * 3 + dir, flip)
        }
    }
}

fn sprite_row(unit: &Unit, wobble: usize) -> usize {
    let data = lookup_unit(unit.kind());
    match data.sprite.atlas {
        Atlas::Sprites | Atlas::Words => data.sprite.row,
        Atlas::Tiled => data.sprite.row + wobble,
    }
}

fn sprite_facing(direction: Direction, facing: Facing) -> (usize, bool) {
    match facing {
        Facing::Fixed => (0, false),
        Facing::Directional => (direction_index(direction), direction == Direction::West),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        sprite::{direction_index, sprite_facing, sprite_row},
        unit::{
            Facing, Noun, Unit,
            UnitKind::{self},
        },
        world::Direction,
    };

    #[test]
    fn direction_index_returns_correct_index() {
        assert_eq!(0, direction_index(Direction::East));
        assert_eq!(0, direction_index(Direction::West));
        assert_eq!(2, direction_index(Direction::North));
        assert_eq!(1, direction_index(Direction::South));
    }

    #[test]
    fn sprite_non_tiled_units_do_not_add_wobble() {
        let row = sprite_row(&Unit::new(0, UnitKind::Object(Noun::Baba), Direction::East), 1);
        assert_eq!(0, row);
    }

    #[test]
    fn sprite_tiled_units_add_wobble() {
        let row = sprite_row(&Unit::new(0, UnitKind::Object(Noun::Wall), Direction::East), 1);
        assert_eq!(11, row);
    }

    #[test]
    fn directional_units_are_flipped() {
        let (_, flip) = sprite_facing(Direction::West, Facing::Directional);
        assert!(flip)
    }

    #[test]
    fn non_directional_units_are_not_flipped() {
        let (_, flip) = sprite_facing(Direction::West, Facing::Fixed);
        assert!(!flip)
    }
}
