use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{pass_data_internal, DataBuffer, DataSink, DataSource};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::prelude::{Bundle, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

#[derive(Component)]
pub struct Splitter {
    throughput: f32,
}

impl Splitter {
    pub fn get_bundle(
        position: GridPosition,
        throughput: f32,
        source_dir: Direction,
        source_count: i8,
    ) -> impl Bundle {
        (
            Splitter { throughput },
            position,
            Tiles::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                    for i in 0..source_count {
                        spawner.spawn((
                            GridSprite(Color::linear_rgba(0.1, 0.3, 1.0, 1.0)),
                            DataSource {
                                direction: source_dir,
                                throughput,
                                limited: true,
                                buffer: DataBuffer::default(),
                            },
                            position.offset(source_dir.rotate_counterclockwise(), 1)
                        ));
                    }
                    spawner.spawn((
                        DataSink {
                            direction: source_dir.opposite(),
                            buffer: DataBuffer::default(),
                        },
                        position,
                        GridSprite(Color::linear_rgba(0.1, 0.3, 1.0, 1.0)),
                        Text2d::default(),
                    ));
                },
            )),
        )
    }
}

pub fn do_splitting(
    splitters: Query<(&Splitter, &Tiles)>,
    mut sinks: Query<(Entity, &mut DataSink)>,
    mut sources: Query<(Entity, &mut DataSource)>,
    time: Res<Time>,
) {
    for (splitter, tiles) in splitters {
        let Some((_, mut sink)) = sinks.iter_mut().find(|(entity, _)| tiles.contains(entity))
        else {
            continue;
        };

        let mut iter = sources
            .iter_mut()
            .filter(|(entity, _)| tiles.contains(entity));

        let (Some((_, mut source1)), Some((_, mut source2))) = (iter.next(), iter.next()) else {
            continue;
        };

        let shape = &sink.buffer.shape;
        if shape.is_some() {
            source1.buffer.set_shape(shape.as_ref());
            source2.buffer.set_shape(shape.as_ref());

            let amount = (sink.buffer.value / 2.).min(splitter.throughput / 2. * time.delta_secs());

            pass_data_internal(&mut source1, &mut sink, amount);
            pass_data_internal(&mut source2, &mut sink, amount);
        }
    }
}
