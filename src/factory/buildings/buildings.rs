use bevy::prelude::*;

#[derive(Clone)]
pub enum BuildingSpecificData {
    Collector {
        collection_rate: f32,
        collector_type: String,
    },
    Aggregator {
        loss_rate: f32,
        speed: f32,
    },
    Link {
        throughput: f32,
    },
    Splitter {
        outputs: i32,
        loss_rate: f32,
    },
    Combiner {
        inputs: i32,
        loss_rate: f32,
    },
    Decoupler {
        outputs: i32,
        loss_rate: f32,
    },
}

#[derive(Clone)]
pub struct BuildingData {
    // Common UI fields
    pub sprite_path: String,
    pub grid_width: i64,
    pub grid_height: i64,
    pub cost: i32,
    pub name: String,
    // Specific gameplay attributes
    pub specific: BuildingSpecificData,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Collector,
    Aggregator,
    Link,
    Splitter2x1,
    Splitter3x1,
    Splitter4x1,
    Combiner,
    Decoupler,
}

impl BuildingType {
    pub fn data(&self) -> BuildingData {
        match self {
            BuildingType::Collector => BuildingData {
                sprite_path: "buildings/collector.png".to_string(),
                grid_width: 1,
                grid_height: 1,
                cost: 50,
                name: "Collector".to_string(),
                specific: BuildingSpecificData::Collector {
                    collection_rate: 5.0,
                    collector_type: "Resource".to_string(),
                },
            },
            BuildingType::Aggregator => BuildingData {
                sprite_path: "buildings/aggregator.png".to_string(),
                grid_width: 1,
                grid_height: 1,
                cost: 75,
                name: "Aggregator".to_string(),
                specific: BuildingSpecificData::Aggregator {
                    loss_rate: 0.1,
                    speed: 2.0,
                },
            },
            BuildingType::Link => BuildingData {
                sprite_path: "buildings/link.png".to_string(),
                grid_width: 1,
                grid_height: 1,
                cost: 25,
                name: "Link".to_string(),
                specific: BuildingSpecificData::Link { throughput: 10.0 },
            },
            BuildingType::Splitter2x1 => BuildingData {
                sprite_path: "buildings/splitter_2x1.png".to_string(),
                grid_width: 2,
                grid_height: 1,
                cost: 60,
                name: "Splitter 2x1".to_string(),
                specific: BuildingSpecificData::Splitter {
                    outputs: 2,
                    loss_rate: 0.05,
                },
            },
            BuildingType::Splitter3x1 => BuildingData {
                sprite_path: "buildings/splitter_3x1.png".to_string(),
                grid_width: 3,
                grid_height: 1,
                cost: 90,
                name: "Splitter 3x1".to_string(),
                specific: BuildingSpecificData::Splitter {
                    outputs: 3,
                    loss_rate: 0.05,
                },
            },
            BuildingType::Splitter4x1 => BuildingData {
                sprite_path: "buildings/splitter_4x1.png".to_string(),
                grid_width: 4,
                grid_height: 1,
                cost: 120,
                name: "Splitter 4x1".to_string(),
                specific: BuildingSpecificData::Splitter {
                    outputs: 4,
                    loss_rate: 0.05,
                },
            },
            BuildingType::Combiner => BuildingData {
                sprite_path: "buildings/combiner.png".to_string(),
                grid_width: 1,
                grid_height: 1,
                cost: 80,
                name: "Combiner".to_string(),
                specific: BuildingSpecificData::Combiner {
                    inputs: 2,
                    loss_rate: 0.05,
                },
            },
            BuildingType::Decoupler => BuildingData {
                sprite_path: "buildings/building_placeholder.png".to_string(),
                grid_width: 1,
                grid_height: 1,
                cost: 70,
                name: "Decoupler".to_string(),
                specific: BuildingSpecificData::Decoupler {
                    outputs: 3,
                    loss_rate: 0.1,
                },
            },
        }
    }
}
