use crate::factory::buildings::aggregator::Aggregator;
use crate::factory::buildings::combiner::Combiner;
use crate::factory::buildings::delinker::Delinker;
use crate::factory::buildings::sink::SinkBuilding;
use crate::factory::buildings::source::SourceBuilding;
use crate::factory::buildings::splitter::Splitter;
use crate::factory::buildings::trunker::Trunker;
use crate::factory::logical::{BasicDataType, DataAttribute, Dataset};
use crate::factory::physical::PhysicalLink;
use crate::grid::{Direction, GridPosition};
use bevy::math::I64Vec2;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::Commands;

pub fn spawn_combiner_test(commands: &mut Commands) {
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: -5, y: 1 }),
        vec![Direction::Right],
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
        5.0,
        false,
    ));
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: -5, y: 2 }),
        vec![Direction::Right],
        Dataset {
            contents: HashMap::from([(BasicDataType::Biometric, HashSet::<DataAttribute>::new())]),
        },
        5.0,
        false,
    ));
    commands.spawn(Combiner::get_bundle(
        GridPosition(I64Vec2 { x: -4, y: 1 }),
        5.0,
        Direction::Right,
        2,
    ));
    commands.spawn(SinkBuilding::get_bundle(
        GridPosition(I64Vec2 { x: -3, y: 1 }),
        vec![Direction::Left],
        None,
    ));
}
pub fn spawn_sized_sink_test(commands: &mut Commands) {
    commands.spawn(SinkBuilding::get_sized_bundle(
        GridPosition(I64Vec2::new(0, -10)),
        2,
        None,
    ));
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: -1, y: -10 }),
        vec![Direction::Right],
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
        10.0,
        false,
    ));
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: 0, y: -11 }),
        vec![Direction::Up],
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
        100.0,
        false,
    ));
}
pub fn spawn_trunking_test(commands: &mut Commands) {
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: -5, y: 5 }),
        vec![Direction::Right],
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
        100.0,
        false,
    ));
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: -5, y: 6 }),
        vec![Direction::Right],
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
        5.0,
        false,
    ));
    commands.spawn(Trunker::get_bundle(
        GridPosition(I64Vec2 { x: -4, y: 5 }),
        10.0,
        Direction::Right,
        2,
    ));
    commands.spawn(SinkBuilding::get_bundle(
        GridPosition(I64Vec2 { x: -3, y: 5 }),
        vec![Direction::Left],
        None,
    ));
}

pub fn spawn_delinker_test(commands: &mut Commands) {
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: 0, y: 1 + 5 }),
        vec![Direction::Right],
        Dataset {
            contents: HashMap::from([
                (BasicDataType::Behavioural, HashSet::<DataAttribute>::new()),
                (BasicDataType::Biometric, HashSet::<DataAttribute>::new()),
            ]),
        },
        5.0,
        false,
    ));
    commands.spawn(Aggregator::get_bundle(
        GridPosition(I64Vec2 { x: 1, y: 1 + 5 }),
        1.0,
        Direction::Right,
    ));
    // commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I64Vec2 {
    //     x: 1,
    //     y: 1,
    // })));
    commands.spawn(PhysicalLink::get_bundle(GridPosition(I64Vec2 {
        x: 2,
        y: 1 + 5,
    })));
    commands.spawn(Delinker::get_bundle(
        GridPosition(I64Vec2 { x: 3, y: 1 + 5 }),
        50.,
        Direction::Right, /* f32 */
        /* grid::Direction */
        2,
    ));
    commands.spawn(PhysicalLink::get_bundle(GridPosition(I64Vec2 {
        x: 4,
        y: 1 + 5,
    })));
    commands.spawn(PhysicalLink::get_bundle(GridPosition(I64Vec2 {
        x: 4,
        y: 2 + 5,
    })));
    commands.spawn(SinkBuilding::get_bundle(
        GridPosition(I64Vec2 { x: 5, y: 1 + 5 }),
        vec![Direction::Left],
        None,
    ));
    commands.spawn(SinkBuilding::get_bundle(
        GridPosition(I64Vec2 { x: 5, y: 2 + 5 }),
        vec![Direction::Left],
        None,
    ));
}

pub fn spawn_splitter_test(commands: &mut Commands) {
    commands.spawn(SourceBuilding::get_bundle(
        GridPosition(I64Vec2 { x: 0, y: 1 }),
        vec![Direction::Right],
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
        5.0,
        false,
    ));
    commands.spawn(Aggregator::get_bundle(
        GridPosition(I64Vec2 { x: 1, y: 1 }),
        1.0,
        Direction::Right,
    ));
    // commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I64Vec2 {
    //     x: 1,
    //     y: 1,
    // })));
    commands.spawn(PhysicalLink::get_bundle(GridPosition(I64Vec2 {
        x: 2,
        y: 1,
    })));
    commands.spawn(Splitter::get_bundle(
        GridPosition(I64Vec2 { x: 3, y: 1 }),
        50.,
        Direction::Right, /* f32 */
        /* grid::Direction */
        2,
    ));
    commands.spawn(PhysicalLink::get_bundle(GridPosition(I64Vec2 {
        x: 4,
        y: 1,
    })));
    commands.spawn(PhysicalLink::get_bundle(GridPosition(I64Vec2 {
        x: 4,
        y: 2,
    })));
    commands.spawn(SinkBuilding::get_bundle(
        GridPosition(I64Vec2 { x: 5, y: 1 }),
        vec![Direction::Left],
        None,
    ));
    commands.spawn(SinkBuilding::get_bundle(
        GridPosition(I64Vec2 { x: 5, y: 2 }),
        vec![Direction::Left],
        None,
    ));
}
