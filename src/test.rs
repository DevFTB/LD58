use crate::factory::buildings::aggregator::Aggregator;
use crate::factory::buildings::buildings::Building;
use crate::factory::buildings::splitter::Splitter;
use crate::factory::buildings::{SinkBuilding, SourceBuilding};
use crate::factory::logical::{BasicDataType, DataAttribute, Dataset};
use crate::factory::physical::PhysicalLink;
use crate::grid::{Direction, GridPosition, Orientation};
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
    // Combiner { throughput: 5.0, sink_count: 2 }.spawn(
    //     commands,
    //     GridPosition(I64Vec2 { x: -4, y: 1 }),
    //     Orientation::new(Direction::Right, false),
    // );
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
    // Trunker { threshold_per_sink: 10.0, sink_count: 2 }.spawn(
    //     commands,
    //     GridPosition(I64Vec2 { x: -4, y: 5 }),
    //     Orientation::new(Direction::Right, false),
    // );
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

    Aggregator { throughput: 5.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 1, y: 1 + 5 }),
        Orientation::new(Direction::Right, false),
    );

    PhysicalLink { throughput: 234.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 2, y: 1 + 5 }),
        Orientation::new(Direction::Right, false),
    );

    // Delinker { throughput: 50.0, source_count: 2 }.spawn(
    //     commands,
    //     GridPosition(I64Vec2 { x: 3, y: 1 + 5 }),
    //     Orientation::new(Direction::Right, false),
    // );

    PhysicalLink { throughput: 234.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 4, y: 1 + 5 }),
        Orientation::new(Direction::Right, false),
    );

    PhysicalLink { throughput: 234.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 4, y: 2 + 5 }),
        Orientation::new(Direction::Right, false),
    );

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

    Aggregator { throughput: 1.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 1, y: 1 }),
        Orientation::new(Direction::Right, false),
    );

    PhysicalLink { throughput: 234.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 2, y: 1 }),
        Orientation::new(Direction::Right, false),
    );

    Splitter {
        throughput: 50.0,
        source_count: 3,
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 3, y: 1 }),
        Orientation::new(Direction::Right, false),
    );

    PhysicalLink { throughput: 234.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 4, y: 1 }),
        Orientation::new(Direction::Right, false),
    );

    PhysicalLink { throughput: 234.0 }.spawn(
        commands,
        GridPosition(I64Vec2 { x: 4, y: 2 }),
        Orientation::new(Direction::Right, false),
    );

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
