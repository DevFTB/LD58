use crate::factory::buildings::buildings::{Building, BuildingData, BuildingTypes, SpriteResource};
use crate::factory::buildings::Tiles;
use crate::factory::logical::{
    pass_data_internal, DataAttribute, DataBuffer, DataSink, DataSource,
};
use crate::grid::{GridPosition, GridSprite, Orientation};
use bevy::color::Color;
use bevy::ecs::related;
use bevy::prelude::{Commands, Component, Query, Res, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

#[derive(Component, Clone)]
pub struct Aggregator {
    pub(crate) throughput: f32,
}

impl Building for Aggregator {
    fn spawn_naked(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        commands
            .spawn((
                position,
                related!(
                    Tiles[
                    (
                        DataSink {
                            direction: orientation.direction.opposite(),
                            buffer: DataBuffer::default(),
                        },
                        position,
                        GridSprite(Color::linear_rgba(1.0, 0.0, 1.0, 0.3)),
                        Text2d::default(),
                    ),
                    (
                        DataSource {
                            direction: orientation.direction,
                            throughput: self.throughput,
                            limited: true,
                            buffer: DataBuffer::default()
                        },
                        position,
                        GridSprite(Color::linear_rgba(1.0, 0.0, 1.0, 0.3)),
                    )
                ]),
                self.clone(),
            ))
            .id()
    }

    fn data(&self) -> BuildingData {
        BuildingData {
            sprite: SpriteResource::Atlas(1),
            grid_width: 1,
            grid_height: 1,
            cost: 75,
            name: "Aggregator".to_string(),
            building_type: BuildingTypes::Aggregator(Aggregator { throughput: 5.0 }),
        }
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
            source.buffer.set_shape(aggregated_shape.as_ref());
            pass_data_internal(&mut source, &mut sink, agg.throughput * time.delta_secs());
        }
    }
}
