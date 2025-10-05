use crate::factory::buildings::Tiles;
use crate::factory::logical::{
    pass_data_internal, DataAttribute, DataBuffer, DataSink, DataSource,
};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::related;
use bevy::prelude::{Bundle, Component, Query, Res, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

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
            related!(
                Tiles[
                (
                    DataSink {
                        direction: output_direction.opposite(),
                        buffer: DataBuffer::default(),
                    },
                    position,
                    Text2d::default(),
                ),
                (
                    DataSource {
                        direction: output_direction,
                        throughput,
                        limited: true,
                        buffer: DataBuffer::default()
                    },
                    position
                )
            ]),
        )
    }
}

pub fn do_aggregation(
    aggregators: Query<(&Aggregator, &Tiles)>,
    mut sinks: Query<(Entity, &mut DataSink)>,
    mut sources: Query<(Entity, &mut DataSource)>,
    time: Res<Time>,
) {
    for (agg, tiles) in aggregators {
        let Some((_, mut sink)) = sinks.iter_mut().find(|(entity, _)| tiles.contains(entity))
        else {
            continue;
        };
        let Some((_, mut source)) = sources
            .iter_mut()
            .find(|(entity, _)| tiles.contains(entity))
        else {
            continue;
        };

        let aggregated_shape = sink
            .buffer
            .shape
            .as_ref()
            .map(|ds| ds.clone().with_attribute(DataAttribute::Aggregated));

        if aggregated_shape.is_some() {
            source.buffer.set_shape(&aggregated_shape);
            pass_data_internal(&mut source, &mut sink, agg.throughput * time.delta_secs());
        }
    }
}
