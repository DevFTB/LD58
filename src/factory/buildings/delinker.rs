use crate::factory::buildings::buildings::{Building, BuildingData, BuildingTypes, SpriteResource};
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink, DataSource, Dataset};
use crate::grid::{GridPosition, GridSprite, Orientation};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

#[derive(Component, Clone)]
pub struct Delinker {
    pub(crate) throughput: f32,
    pub(crate) source_count: i64,
}

impl Building for Delinker {
    fn spawn(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        let source_count = self.source_count;
        let throughput = self.throughput;
        commands
            .spawn((
                position,
                Tiles::spawn(SpawnWith(
                    move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                        spawner.spawn((
                            DataSink {
                                direction: orientation.direction.opposite(),
                                buffer: DataBuffer::default(),
                            },
                            position,
                            GridSprite(Color::linear_rgba(1.0, 0.5, 0.0, 0.3)),
                            Text2d::default(),
                        ));
                        for i in 0..source_count {
                            spawner.spawn((
                                GridSprite(Color::linear_rgba(1.0, 0.5, 0.0, 0.3)),
                                DataSource {
                                    direction: orientation.direction,
                                    throughput,
                                    limited: true,
                                    buffer: DataBuffer::default(),
                                },
                                position.offset(orientation.layout_direction(), i as i64),
                            ));
                        }
                    },
                )),
                self.clone(),
            ))
            .id()
    }

    fn data(&self) -> BuildingData {
        BuildingData {
            sprite: SpriteResource::Atlas(self.source_count as usize + 7),
            grid_width: self.source_count,
            grid_height: 1,
            cost: 60,
            name: format!("Delinker {}x1", self.source_count),
            building_type: BuildingTypes::Delinker(Delinker {
                source_count: self.source_count,
                throughput: 5.0,
            }),
        }
    }
}

pub fn do_delinking(
    splitters: Query<(&Delinker, &Tiles)>,
    mut sinks: Query<(Entity, &mut DataSink)>,
    mut sources: Query<(Entity, &mut DataSource)>,
    time: Res<Time>,
) {
    for (splitter, tiles) in splitters {
        let Some((_, mut sink)) = sinks.iter_mut().find(|(entity, _)| tiles.contains(entity))
        else {
            continue;
        };

        let Some(shape) = &sink.buffer.shape else {
            continue;
        };

        let process_amount = (splitter.throughput * time.delta_secs()).min(sink.buffer.value);

        let sources = sources
            .iter_mut()
            .sort_by_key::<Entity, _>(|&entity| entity)
            .filter(|(entity, _source)| tiles.contains(&entity))
            .collect::<Vec<_>>();

        let mut entries = shape.contents.iter().collect::<Vec<_>>();

        if entries.len() != sources.len() {
            continue;
        }

        entries.sort_by_key(|e| e.0);
        let datasets = entries
            .into_iter()
            .map(|(data_type, attr_set)| Dataset {
                contents: HashMap::from([(data_type.clone(), attr_set.clone())]),
            })
            .collect::<Vec<_>>();

        for (ds, (_, mut source)) in datasets.iter().zip(sources) {
            source.buffer.add(ds, process_amount);
        }

        sink.buffer.remove(process_amount);
    }
}
