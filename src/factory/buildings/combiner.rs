use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{
    BasicDataType, DataAttribute, DataBuffer, DataSink, DataSource, Dataset,
};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Bundle, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;
use std::hash::Hash;

#[derive(Component)]
pub struct Combiner {
    throughput: f32,
}

impl Combiner {
    pub fn get_bundle(
        position: GridPosition,
        throughput: f32,
        source_dir: Direction,
        sink_count: i8,
    ) -> impl Bundle {
        (
            Combiner { throughput },
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
                            throughput,
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
pub fn get_disjoint_data<'a, I>(
    mut datasets: I,
) -> Option<HashMap<BasicDataType, HashSet<DataAttribute>>>
where
    I: Iterator<Item = &'a Dataset>,
    BasicDataType: Clone + Eq + Hash,
    DataAttribute: Clone + Eq + Hash,
{
    // The accumulator is now a HashMap, which will be our final result if successful.
    datasets.try_fold(HashMap::new(), |mut acc, dataset| {
        // We iterate over the key-value pairs of the current dataset's contents.
        for (key, attributes) in &dataset.contents {
            // `insert` returns `None` if the key was new, or `Some(old_value)` if the
            // key already existed. The `is_some()` check is a clean way to detect an overlap.
            if acc.insert(key.clone(), attributes.clone()).is_some() {
                // Overlap detected! The key was already in our accumulator.
                // Short-circuit by returning None.
                return None;
            }
        }
        // No conflict in this dataset, continue with the updated accumulator.
        Some(acc)
    })
}
pub fn do_combining(
    combiners: Query<(&Combiner, &Tiles)>,
    mut sinks: Query<(Entity, &mut DataSink)>,
    mut sources: Query<(Entity, &mut DataSource)>,
    time: Res<Time>,
) {
    for (combiner, tiles) in combiners {
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

        // Make sure all the datasets in every sink have disjoint BasicDataTypes
        let Some(disjoint_data) =
            get_disjoint_data(sinks.iter().filter_map(|s| s.buffer.shape.as_ref()))
        else {
            continue;
        };
        let smallest_buffer_amount = sinks.iter().map(|s| s.buffer.value).reduce(f32::min);
        let process_amount = smallest_buffer_amount
            .map_or(0., |sba| sba.min(time.delta_secs() * combiner.throughput));

        source.buffer.add(
            &Dataset {
                contents: disjoint_data,
            },
            process_amount,
        );
        sinks
            .iter_mut()
            .for_each(|s| s.buffer.remove(process_amount));
    }
}
