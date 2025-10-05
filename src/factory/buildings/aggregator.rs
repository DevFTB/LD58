use crate::factory::logical::{
    pass_data_internal, DataAttribute, DataBuffer, DataSink, DataSource,
};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::children;
use bevy::prelude::{Bundle, Children, Component, Query, Res, Time};
use bevy::prelude::{Entity, SpawnRelated};

#[derive(Component)]
pub struct Aggregator {
    throughput: f32,
}

impl Aggregator {
    pub fn get_bundle(
        position: GridPosition,
        throughput: f32,
        output_direction: Direction,
    ) -> impl Bundle {
        (
            Aggregator { throughput },
            position,
            GridSprite(Color::linear_rgba(1.0, 0.0, 1.0, 1.0)),
            children![
                (
                    DataSink {
                        input_direction: output_direction.opposite(),
                        buffer: DataBuffer::default(),
                    },
                    position,
                ),
                (
                    DataSource {
                        output_direction,
                        throughput,
                        limited: true,
                        buffer: DataBuffer::default()
                    },
                    position
                )
            ],
        )
    }
}

pub fn do_aggregation(
    aggregators: Query<(&Aggregator, &Children)>,
    mut sinks: Query<(Entity, &mut DataSink)>,
    mut sources: Query<(Entity, &mut DataSource)>,
    time: Res<Time>,
) {
    for (agg, children) in aggregators {
        let Some((_, mut sink)) = sinks
            .iter_mut()
            .find(|(entity, _)| children.contains(entity))
        else {
            continue;
        };
        let Some((_, mut source)) = sources
            .iter_mut()
            .find(|(entity, _)| children.contains(entity))
        else {
            continue;
        };

        let aggregated_shape = sink
            .buffer
            .shape
            .as_ref()
            .map(|ds| ds.clone().with_attribute(DataAttribute::Aggregated));

        if aggregated_shape.is_some() {
            println!("Setting sink shape from aggregated shape");
            source.buffer.set_shape(&aggregated_shape);
            pass_data_internal(&mut source, &mut sink, agg.throughput, time.delta_secs());
        }
    }
}
