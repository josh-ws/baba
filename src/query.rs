use crate::{
    rule::{Complement, Rules},
    unit::{Noun, Operator, Property, Unit},
    world::{Cell, Direction, Grid, Pos},
};

#[derive(Debug)]
pub struct UnitRef {
    pub unit_id: u64,
    pub pos: Pos, // position at the point of query. May be stale
}

impl UnitRef {
    fn new(unit_id: u64, pos: Pos) -> Self {
        Self { unit_id, pos }
    }
}

pub struct CreateUnitRef {
    pub unit_id: u64,
    pub pos: Pos,
    pub into_noun: Noun,
    pub direction: Direction, // inherited from the source unit
}

impl CreateUnitRef {
    fn new(unit_id: u64, pos: Pos, into_noun: Noun, direction: Direction) -> Self {
        Self {
            unit_id,
            pos,
            into_noun,
            direction,
        }
    }
}

/// query the grid for A HAS B where A is in `doomed`. All new units created by
/// HAS will be returned along with their position.
pub fn query_has(rules: &Rules, grid: &Grid, doomed: &[u64]) -> Vec<CreateUnitRef> {
    let mut result = Vec::new();
    let items = doomed.iter().filter_map(|id| grid.find_unit(*id));
    for (pos, unit) in items {
        let create = rules.unit_has(unit.noun());
        for created in create {
            result.push(CreateUnitRef::new(unit.id(), pos, created, unit.direction()));
        }
    }
    result
}

/// query the grid for all A IS B and return all units to be transformed
pub fn query_is_noun(rules: &Rules, grid: &Grid) -> Vec<CreateUnitRef> {
    let transforms = rules
        .iter()
        .filter_map(|r| match r.complement {
            Complement::Noun(n) if r.operator == Operator::Is => Some((r.subject, n)),
            _ => None,
        })
        .collect::<Vec<(Noun, Noun)>>();

    let mut result = Vec::new();
    for (pos, unit) in grid.units_with_pos() {
        let is_self = transforms.iter().any(|(from, to)| *from == unit.noun() && from == to);
        if is_self {
            continue; // e.g. BABA IS BABA. Skip transforms
        }
        for (_, to) in transforms.iter().filter(|(from, _)| *from == unit.noun()) {
            result.push(CreateUnitRef::new(unit.id(), pos, *to, unit.direction()));
        }
    }
    result
}

/// query the grid for all `A` where A IS <PROP>
pub fn query_is_property(rules: &Rules, grid: &Grid, prop: Property) -> Vec<UnitRef> {
    grid.units_with_pos()
        .filter(|(_, unit)| rules.unit_has_prop(unit.noun(), prop))
        .map(|(pos, unit)| UnitRef::new(unit.id(), pos))
        .collect::<Vec<UnitRef>>()
}

/// query the grid for all `A` and `B` where A and B share a cell, and A|B IS SINK
pub fn query_sink(rules: &Rules, grid: &Grid) -> Vec<UnitRef> {
    query_layers(
        rules,
        grid,
        |r, layer| layer.len() > 1 && layer_has(r, layer, &[Property::Sink]),
        |_, _| true,
    )
}

/// query the grid for all `A` that shares a cell with some `B`, where A IS YOU and B IS DEFEAT
/// in other words, all `YOU` units that should be defeated
pub fn query_defeat(rules: &Rules, grid: &Grid) -> Vec<UnitRef> {
    query_layers(
        rules,
        grid,
        |r, layer| layer_has(r, layer, &[Property::Defeat]),
        |r, u| r.unit_has_prop(u.noun(), Property::You),
    )
}

/// query the grid for all `A` that shares a cell with some `B`, where A IS MELT and B IS HOT
/// all units that should melt this turn
pub fn query_melt(rules: &Rules, grid: &Grid) -> Vec<UnitRef> {
    query_layers(
        rules,
        grid,
        |r, layer| layer_has(r, layer, &[Property::Hot]),
        |r, u| r.unit_has_prop(u.noun(), Property::Melt),
    )
}

/// query the grid for all nouns `A` overlapping `B` where B IS SELECT
pub fn query_selected(rules: &Rules, grid: &Grid) -> Vec<UnitRef> {
    query_cells(
        rules,
        grid,
        |r, c| cell_has(r, c, &[Property::Select]),
        |r, u| u.is_object() && !r.unit_has_prop(u.noun(), Property::Select),
    )
}

/// helper function. filter down the cells, then the units in those cells.
/// return all matching units.
pub fn query_cells(
    rules: &Rules,
    grid: &Grid,
    cell_pred: impl Fn(&Rules, &Cell) -> bool,
    unit_pred: impl Fn(&Rules, &Unit) -> bool,
) -> Vec<UnitRef> {
    grid.cells_with_pos()
        .filter(|(_, cell)| cell_pred(rules, cell))
        .flat_map(|(pos, cell)| cell.units().iter().map(move |u| (pos, u)))
        .filter(|(_, unit)| unit_pred(rules, unit))
        .map(|(pos, unit)| UnitRef::new(unit.id(), pos))
        .collect::<Vec<UnitRef>>()
}

/// like query_cells, but split the cell into its float layers for the check
/// for example, a FLOAT BABA and a non-FLOAT FLAG should not interact in a win check.
pub fn query_layers(
    rules: &Rules,
    grid: &Grid,
    layer_pred: impl Fn(&Rules, &[&Unit]) -> bool,
    unit_pred: impl Fn(&Rules, &Unit) -> bool,
) -> Vec<UnitRef> {
    grid.cells_with_pos()
        .flat_map(|(pos, cell)| layers(rules, cell).map(move |layer| (pos, layer)))
        .filter(|(_, layer)| layer_pred(rules, layer))
        .flat_map(|(pos, layer)| layer.into_iter().map(move |u| (pos, u)))
        .filter(|(_, unit)| unit_pred(rules, unit))
        .map(|(pos, unit)| UnitRef::new(unit.id(), pos))
        .collect()
}

fn cell_has_prop(rules: &Rules, cell: &Cell, prop: Property) -> bool {
    cell.units().iter().any(|u| rules.unit_has_prop(u.noun(), prop))
}

/// query the specified cell and check if any unit satisifes all props in `props`
pub fn cell_has(rules: &Rules, cell: &Cell, props: &[Property]) -> bool {
    props.iter().all(|p| cell_has_prop(rules, cell, *p))
}

/// like cell_has, but on the float layers instead
pub fn layer_has(rules: &Rules, layer: &[&Unit], props: &[Property]) -> bool {
    props
        .iter()
        .all(|p| layer.iter().any(|u| rules.unit_has_prop(u.noun(), *p)))
}

pub fn any_layer_has(rules: &Rules, grid: &Grid, props: &[Property]) -> bool {
    grid.cells()
        .iter()
        .flat_map(|c| layers(rules, c))
        .any(|layer| layer_has(rules, &layer, props))
}

fn layers<'a>(rules: &Rules, cell: &'a Cell) -> impl Iterator<Item = Vec<&'a Unit>> {
    let (float, nonfloat): (Vec<&Unit>, Vec<&Unit>) = cell
        .units()
        .iter()
        .partition(|u| rules.unit_has_prop(u.noun(), Property::Float));
    [nonfloat, float].into_iter().filter(|l| !l.is_empty())
}
