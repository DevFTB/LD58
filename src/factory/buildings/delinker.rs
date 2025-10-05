use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink, DataSource, Dataset};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Bundle, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

#[derive(Component)]
pub struct Delinker {
    throughput: f32,
}

impl Delinker {
    pub fn get_bundle(
        position: GridPosition,
        throughput: f32,
        source_dir: Direction,
        source_count: i8,
    ) -> impl Bundle {
        (
            Delinker { throughput },
            position,
            Tiles::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                    for i in 0..source_count {
                        spawner.spawn((
                            DataSource {
                                direction: source_dir.clone(),
                                throughput,
                                limited: true,
                                buffer: DataBuffer::default(),
                            },
                            position.offset(Direction::Up, i as i64),
                            GridSprite(Color::linear_rgba(0.7, 0.3, 1.0, 1.0)),
                        ));
                    }
                    spawner.spawn((
                        DataSink {
                            direction: source_dir.opposite(),
                            buffer: DataBuffer::default(),
                        },
                        position,
                        Text2d::default(),
                    ));
                },
            )),
        )
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
