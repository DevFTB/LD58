use crate::factory::buildings::buildings::{Building, BuildingData, SpriteResource};
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{
    BasicDataType, DataAttribute, DataBuffer, DataSink, DataSource, Dataset,
};
use crate::grid::{GridPosition, GridSprite, Orientation};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Commands, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;
use std::hash::Hash;

#[derive(Component, Clone)]
pub struct Combiner {
    pub(crate) throughput: f32,
    pub(crate) sink_count: i64,
}

impl Building for Combiner {
    fn spawn_naked(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        let sink_count = self.sink_count;
        let throughput = self.throughput;
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
                                direction: orientation.direction,
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
            sprite: SpriteResource::Atlas(self.sink_count as usize + 4),
            grid_width: self.sink_count,
            grid_height: 1,
            cost: 60,
            name: format!("Combiner {}x1", self.sink_count),
        }
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
