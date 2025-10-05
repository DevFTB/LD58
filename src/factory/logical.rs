use crate::grid::Direction;
use bevy::prelude::{DetectChanges, Query, Ref, Res};
use bevy::sprite::Text2d;
use bevy::time::Time;
use bevy::{
    ecs::{component::Component, entity::Entity},
    platform::collections::{HashMap, HashSet},
};
use core::fmt;
use std::fmt::Write;

// The fundamental types of data
#[derive(Component, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum BasicDataType {
    Biometric,   // A
    Economic,    // B
    Behavioural, // C
    Telemetry,   // D
}

impl BasicDataType {
    pub(crate) fn to_shorthand(&self) -> &str {
        match self {
            BasicDataType::Biometric => "A",
            BasicDataType::Economic => "B",
            BasicDataType::Behavioural => "C",
            BasicDataType::Telemetry => "D",
        }
    }
}

// Attributes that modify a data stream
#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DataAttribute {
    Aggregated,
    DeIdentified,
    Cleaned,
    Illegal,
}

impl DataAttribute {
    pub(crate) fn to_shorthand(&self) -> &str {
        match self {
            DataAttribute::Aggregated => "+",
            DataAttribute::DeIdentified => "-",
            DataAttribute::Cleaned => "*",
            DataAttribute::Illegal => "$",
        }
    }
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
#[derive(Component, Debug)]
pub struct DataSink {
    pub direction: Direction,
    pub buffer: DataBuffer,
}

#[derive(Component, Debug)]
pub struct DataSource {
    pub(crate) direction: Direction,
    pub(crate) throughput: f32,
    pub(crate) buffer: DataBuffer,
    pub(crate) limited: bool,
}

#[derive(Default, Debug)]
pub struct DataBuffer {
    pub(crate) shape: Option<Dataset>,
    pub(crate) value: f32,
}
impl fmt::Display for DataBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_string = self.shape.as_ref().map_or(String::from("None"), |shape| {
            shape
                .contents
                .iter()
                .flat_map(|(k, v)| {
                    [
                        k.to_shorthand().to_string(),
                        v.iter()
                            .map(|attr| attr.to_shorthand())
                            .collect::<Vec<_>>()
                            .join("")
                            .to_string(),
                    ]
                })
                .collect::<Vec<_>>()
                .join("")
                .to_string()
        });
        write!(f, "{}: {}", type_string, self.value.round())
    }
}
impl DataBuffer {
    pub(crate) fn set_shape(&mut self, p0: Option<&Dataset>) {
        let are_different = if let (Some(s1), Some(s2)) = (self.shape.as_ref(), p0) {
            // Case 1: Both are Some. Compare their values by dereferencing.
            *s1 != *s2
        } else {
            // Case 2: One is Some and the other is None.
            // `is_some()` will be different (true vs false), so they are not equal.
            // If both are None, `is_some()` is the same (false vs false), so they are equal.
            self.shape.is_some() != p0.is_some()
        };

        if are_different {
            println!("{:?} != {:?}", self.shape, &p0);
            self.shape = p0.cloned();
            self.value = 0.;
        }
    }

    pub(crate) fn add(&mut self, dataset: &Dataset, amount: f32) {
        self.set_shape(Some(dataset));

        self.value += amount;
    }
    pub(crate) fn remove(&mut self, amount: f32) {
        self.value = (self.value - amount).max(0.);
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
            text.0 = format!("{}", sink.buffer);
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
    sink.buffer.set_shape(source.buffer.shape.as_ref());
    let packet = if source.limited {
        source.buffer.value.clamp(0., source.throughput * secs)
    } else {
        source.throughput * secs
    };

    source.buffer.value = (source.buffer.value - packet).max(0.);
    sink.buffer.value += packet;
}
pub fn pass_data_internal(source: &mut DataSource, sink: &mut DataSink, amount: f32) {
    let amount = amount.min(sink.buffer.value);
    source.buffer.value += amount;
    sink.buffer.value -= amount;
}
