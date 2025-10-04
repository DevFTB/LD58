use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::prelude::{Deref, Query, With};
use bevy::{
    color::Color,
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    platform::collections::{HashMap, HashSet},
};

#[derive(Component, Default, Debug)]
pub struct FactoryTile;

#[derive(Component, Default, Debug)]
#[require(FactoryTile)]
pub struct Locked;

// The fundamental types of data
#[derive(Component, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum BasicDataType {
    Biometric,   // A
    Economic,    // B
    Behavioural, // C
    Telemetry,   // D
}

// Attributes that modify a data stream
#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DataAttribute {
    Aggregated,
    DeIdentified,
    Cleaned,
    Illegal,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Dataset {
    // The core of the data packet.
    // Maps each data type present in the packet to a set of its attributes.
    pub contents: HashMap<BasicDataType, HashSet<DataAttribute>>,
}
#[derive(Component, Deref)]
pub struct DataInput(Direction);

#[derive(Component, Deref)]
pub struct DataOutput(Direction);

#[derive(Component, Default, Deref)]
pub struct DataBuffer(Vec<Dataset>);

// Component for entities that can be connected (inputs/outputs of machines)
#[derive(Component)]
pub struct Source {
    packet: Dataset,
    rate: f32,
}

impl Source {
    pub fn get_spawn_bundle(position: GridPosition, direction: Direction, packet: Dataset, ) -> impl Bundle {
        (
            position,
            Source { packet, rate: 1. },
            DataOutput(direction),
            GridSprite(Color::linear_rgba(0., 1., 0., 1.)),
        )
    }
}

#[derive(Component)]
pub struct Sink {
    packet: Dataset,
}

impl Sink {
    pub fn get_spawn_bundle(position: GridPosition, direction: Direction, packet: Dataset) -> impl Bundle {
        (
            position,
            Sink { packet },
            DataInput(direction),
            GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
        )
    }
}

#[derive(Component)]
pub struct LogicalLink {
    pub links: Vec<Entity>,
    pub(crate) output_entity: Entity,
    pub(crate) input_entity: Entity,
    pub throughput: f32,
}

fn pass_data(mut data_inputs: Query<(&DataBuffer, &LogicalLink), With<DataInput>>, mut data_outputs: Query<&DataOutput>) {
   for (buffer, link) in data_inputs {
        let output= data_outputs.get(link.output_entity).unwrap();

        //*input_buffer.push



   }
}