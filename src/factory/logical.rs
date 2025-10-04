use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::prelude::{Deref, Query, Res, With};
use bevy::{
    color::Color,
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    platform::collections::{HashMap, HashSet},
};
use bevy::time::Time;

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
#[require(DataBuffer)]
pub struct DataInput(Direction);

#[derive(Component, Deref)]
pub struct DataOutput(Direction);

#[derive(Component, Default)]
pub struct DataBuffer {
    shape: Option<Dataset>,
    value: f32,
}

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
            DataBuffer{shape: None, value: 0.},
            GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
        )
    }
}

#[derive(Component, Debug)]
pub struct LogicalLink {
    pub links: Vec<Entity>,
    pub(crate) output_entity: Entity,
    pub(crate) input_entity: Entity,
    pub throughput: f32,
}

pub fn pass_data(data_inputs: Query<(&mut DataBuffer, &LogicalLink), With<DataInput>>, data_outputs: Query<&Source, With<DataOutput>>, time: Res<Time>) {
    println!("{:?}", data_inputs);
   for (mut buffer, link) in data_inputs {
        let source= data_outputs.get(link.output_entity).unwrap();
        if let Some(shape) = &buffer.shape {
            if source.packet != *shape {
                buffer.shape = Some(source.packet.clone());
                buffer.value = 0.;
            }
        } else {
            buffer.shape = Some(source.packet.clone());
            buffer.value = 0.;
        }

        buffer.value += source.rate * time.delta_secs();
   }
}