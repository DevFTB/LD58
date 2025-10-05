use crate::grid::Direction;
use bevy::prelude::{DetectChanges, Query, Ref, Res};
use bevy::sprite::Text2d;
use bevy::time::Time;
use bevy::{
    ecs::{component::Component, entity::Entity},
    platform::collections::{HashMap, HashSet},
};

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

impl Dataset {
    pub fn with_attribute(mut self, attr: DataAttribute) -> Dataset {
        for (_, set) in self.contents.iter_mut() {
            set.insert(attr);
        }

        self
    }
}
#[derive(Component)]
pub struct DataSink {
    pub direction: Direction,
    pub buffer: DataBuffer,
}

#[derive(Component)]
pub struct DataSource {
    pub(crate) direction: Direction,
    pub(crate) throughput: f32,
    pub(crate) buffer: DataBuffer,
    pub(crate) limited: bool,
}

#[derive(Default)]
pub struct DataBuffer {
    pub(crate) shape: Option<Dataset>,
    pub(crate) value: f32,
}

impl DataBuffer {
    pub(crate) fn set_shape(&mut self, p0: &Option<Dataset>) {
        if self.shape != *p0 {
            println!("{:?} != {:?}", self.shape, &p0);
            self.shape = p0.clone();
            self.value = 0.;
        }
    }
}

#[derive(Component, Debug)]
pub struct LogicalLink {
    pub links: Vec<Entity>,
    pub(crate) source: Entity,
    pub(crate) sink: Entity,
    pub throughput: f32,
}
pub fn debug_logical_links(query: Query<Ref<LogicalLink>>) {
    for link in query {
        if link.is_added() {
            println!("Added LogicalLink {:?}", link);
        }
    }
}

pub fn visualise_sinks(query: Query<(Entity, Ref<DataSink>, &mut Text2d)>) {
    for (entity, sink, mut text) in query {
        if sink.is_changed() {
            // println!(
            //     "Sink {:?} storing {:?} of amount {:?}",
            //     entity, sink.buffer.shape, sink.buffer.value
            //
            // );
            text.0 = sink.buffer.value.to_string();
        }
    }
}

pub fn pass_data_system(
    mut sources: Query<&mut DataSource>,
    sinks: Query<(&mut DataSink, &LogicalLink)>,
    time: Res<Time>,
) {
    for (mut sink, link) in sinks {
        let mut source = sources.get_mut(link.source).unwrap();
        pass_data_external(&mut *source, &mut *sink, time.delta_secs());
    }
}
pub fn pass_data_external(source: &mut DataSource, sink: &mut DataSink, secs: f32) {
    sink.buffer.set_shape(&source.buffer.shape);
    let packet = if source.limited {
        source.buffer.value.min(source.throughput * secs)
    } else {
        source.throughput
    };
    sink.buffer.value += packet;
    source.buffer.value -= packet;
}
pub fn pass_data_internal(source: &mut DataSource, sink: &mut DataSink, amount: f32) {
    let amount = amount.min(sink.buffer.value);
    source.buffer.value += amount;
    sink.buffer.value -= amount;
}
