use crate::factory::buildings::buildings::{Building, BuildingData, SpriteResource};
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink, DataSource};
use crate::grid::{GridPosition, GridSprite, Orientation};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::prelude::{Commands, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

#[derive(Component, Clone)]
pub struct Trunker {
    pub(crate) threshold_per_sink: f32,
    pub(crate) sink_count: i64,
}

impl Building for Trunker {
    fn spawn_naked(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        let sink_count = self.sink_count;
        let throughput = self.threshold_per_sink * self.sink_count as f32;
        commands
            .spawn((
                position,
                Tiles::spawn(SpawnWith(
                    move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                        for i in 0..sink_count {
                            spawner.spawn((
                                DataSink {
                                    direction: orientation.direction.opposite(),
                                    buffer: DataBuffer::default(),
                                },
                                Text2d::default(),
                                position.offset(orientation.layout_direction(), i as i64),
                                GridSprite(Color::linear_rgba(0.7, 0.3, 1.0, 0.3)),
                            ));
                        }
                        spawner.spawn((
                            DataSource {
                                throughput,
                                limited: true,
                                direction: orientation.effective_direction(),
                                buffer: DataBuffer::default(),
                            },
                            position,
                            Text2d::default(),
                        ));
                    },
                )),
                self.clone(),
            ))
            .id()
    }

    fn data(&self) -> BuildingData {
        BuildingData {
            sprite: Some(SpriteResource::Atlas(self.sink_count as usize + 10)),
            grid_width: self.sink_count,
            grid_height: 1,
            cost: 60,
            name: format!("Trunker {}x1", self.sink_count),
        }
    }
}
pub fn do_trunking(
    combiners: Query<(&Trunker, &Tiles)>,
    mut sinks: Query<(Entity, &mut DataSink)>,
    mut sources: Query<(Entity, &mut DataSource)>,
    time: Res<Time>,
) {
    for (trunker, tiles) in combiners {
        let Some((_, mut source)) = sources
            .iter_mut()
            .find(|(entity, _)| tiles.contains(entity))
        else {
            continue;
        };

        let mut sinks = sinks
            .iter_mut()
            .filter_map(|(entity, source)| {
                if tiles.contains(&entity) {
                    Some(source)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let any_empty = sinks.iter().any(|s| s.buffer.shape.is_none());
        if any_empty {
            continue;
        }

        let all_have_same_shape = sinks
            .windows(2)
            .all(|w| w[0].buffer.shape == w[1].buffer.shape);
        if all_have_same_shape {
            // Make sure all the datasets in every sink have disjoint BasicDataTypes
            let shape = sinks
                .first()
                .unwrap()
                .buffer
                .shape
                .as_ref()
                .unwrap()
                .clone();

            sinks
                .iter_mut()
                .map(|s| {
                    (
                        s.buffer
                            .value
                            .min(trunker.threshold_per_sink * time.delta_secs()),
                        s,
                    )
                })
                .for_each(|(value, sink)| {
                    sink.buffer.remove(value);
                    source.buffer.add(&shape, value)
                });
        }
    }
}
