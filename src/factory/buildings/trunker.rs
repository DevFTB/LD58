use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink, DataSource};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::prelude::{Bundle, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;
use std::hash::Hash;

#[derive(Component)]
pub struct Trunker {
    threshold_per_sink: f32,
}

impl Trunker {
    pub fn get_bundle(
        position: GridPosition,
        threshold_per_sink: f32,
        source_dir: Direction,
        sink_count: i8,
    ) -> impl Bundle {
        (
            Trunker { threshold_per_sink },
            position,
            Tiles::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                    for i in 0..sink_count {
                        spawner.spawn((
                            DataSink {
                                direction: source_dir.opposite(),
                                buffer: DataBuffer::default(),
                            },
                            Text2d::default(),
                            position.offset(Direction::Up, i as i64),
                            GridSprite(Color::linear_rgba(0.7, 0.3, 1.0, 1.0)),
                        ));
                    }
                    spawner.spawn((
                        DataSource {
                            throughput: threshold_per_sink * 3.,
                            limited: true,
                            direction: source_dir,
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
                    // println!(
                    //     "{} {}",
                    //     value,
                    //     trunker.threshold_per_sink * time.delta_secs()
                    // );
                    sink.buffer.remove(value);
                    source.buffer.add(&shape, value)
                });
        }
    }
}
