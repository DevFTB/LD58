use crate::factory::buildings::aggregator::Aggregator;
use crate::factory::buildings::buildings::Building;
use crate::factory::buildings::combiner::Combiner;
use crate::factory::buildings::delinker::Delinker;
use crate::factory::buildings::sink::SinkBuilding;
use crate::factory::buildings::source::SourceBuilding;
use crate::factory::buildings::splitter::Splitter;
use crate::factory::buildings::trunker::Trunker;
use crate::factory::logical::{BasicDataType, DataAttribute, Dataset};
use crate::factory::physical::PhysicalLink;
use crate::grid::{Direction, GridPosition, Orientation};
use bevy::math::I64Vec2;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::Commands;

pub fn spawn_combiner_test(commands: &mut Commands) {
    SourceBuilding {
        directions: vec![Direction::Right],
        throughput: 5.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -5, y: 1 }),
        Orientation::new(Direction::Up, false),
    );

    SourceBuilding {
        directions: vec![Direction::Right],
        throughput: 5.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([(BasicDataType::Biometric, HashSet::<DataAttribute>::new())]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -5, y: 2 }),
        Orientation::new(Direction::Up, false),
    );

    Combiner {
        throughput: 5.0,
        sink_count: 2,
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -4, y: 1 }),
        Orientation::new(Direction::Right, false),
    );

    SinkBuilding {
        size: I64Vec2::new(1, 1),
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -3, y: 1 }),
        Orientation::new(Direction::Right, false),
    );
}

pub fn spawn_sized_sink_test(commands: &mut Commands) {
    SinkBuilding {
        size: I64Vec2::new(2, 2),
    }
    .spawn(
        commands,
        GridPosition(I64Vec2::new(0, -10)),
        Orientation::new(Direction::Up, false),
    );

    SourceBuilding {
        directions: vec![Direction::Right],
        throughput: 10.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -1, y: -10 }),
        Orientation::new(Direction::Right, false),
    );

    SourceBuilding {
        directions: vec![Direction::Up],
        throughput: 100.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 0, y: -11 }),
        Orientation::new(Direction::Up, false),
    );
}
pub fn spawn_trunking_test(commands: &mut Commands) {
    SourceBuilding {
        directions: vec![Direction::Right],
        throughput: 100.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -5, y: 5 }),
        Orientation::new(Direction::Up, false),
    );

    SourceBuilding {
        directions: vec![Direction::Right],
        throughput: 5.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -5, y: 6 }),
        Orientation::new(Direction::Up, false),
    );

    Trunker {
        threshold_per_sink: 10.0,
        sink_count: 2,
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -4, y: 5 }),
        Orientation::new(Direction::Right, false),
    );

    SinkBuilding {
        size: I64Vec2::new(1, 1),
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: -3, y: 5 }),
        Orientation::new(Direction::Right, false),
    );
}

pub fn spawn_delinker_test(commands: &mut Commands) {
    SourceBuilding {
        directions: vec![Direction::Right],
        throughput: 5.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([
                (BasicDataType::Behavioural, HashSet::<DataAttribute>::new()),
                (BasicDataType::Biometric, HashSet::<DataAttribute>::new()),
            ]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 0, y: 1 + 5 }),
        Orientation::new(Direction::Up, false),
    );

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

    Delinker {
        throughput: 50.0,
        source_count: 2,
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 3, y: 1 + 5 }),
        Orientation::new(Direction::Right, false),
    );

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

    SinkBuilding {
        size: I64Vec2::new(1, 1),
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 5, y: 1 + 5 }),
        Orientation::new(Direction::Right, false),
    );

    SinkBuilding {
        size: I64Vec2::new(1, 1),
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 5, y: 2 + 5 }),
        Orientation::new(Direction::Right, false),
    );
}

pub fn spawn_splitter_test(commands: &mut Commands) {
    SourceBuilding {
        directions: vec![Direction::Right],
        throughput: 5.0,
        limited: false,
        size: I64Vec2::new(1, 1),
        shape: Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 0, y: 1 }),
        Orientation::default(),
    );

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

    SinkBuilding {
        size: I64Vec2::new(1, 1),
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 5, y: 1 }),
        Orientation::new(Direction::Right, false),
    );

    SinkBuilding {
        size: I64Vec2::new(1, 1),
    }
    .spawn(
        commands,
        GridPosition(I64Vec2 { x: 5, y: 2 }),
        Orientation::new(Direction::Right, false),
    );
}
