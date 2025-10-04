use bevy::prelude::*;

#[derive(Clone)]
pub struct BuildingData {
    pub sprite_path: String,
    pub grid_width: i8,
    pub grid_height: i8,
    pub cost: i32,
    pub name: String,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Factory,
    Warehouse,
    Test
}

impl BuildingType {
    pub fn data(&self) -> BuildingData {
        match self {
            BuildingType::Factory => BuildingData {
                sprite_path: "buildings/factory.png".to_string(),
                grid_width: 2,
                grid_height: 1,
                cost: 100,
                name: "Factory".to_string(),
            },
            BuildingType::Warehouse => BuildingData {
                sprite_path: "buildings/warehouse.png".to_string(),
                grid_width: 1,
                grid_height: 2,
                cost: 150,
                name: "Warehouse".to_string(),
            },
            BuildingType::Test => BuildingData {
                sprite_path: "buildings/building_placeholder.png".to_string(),
                grid_width: 1,
                grid_height: 1,
                cost: 50,
                name: "Test Building".to_string(),
            },
        }
    }
}