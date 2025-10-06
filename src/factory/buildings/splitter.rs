use crate::factory::buildings::buildings::{Building, BuildingData, BuildingTypes, SpriteResource};
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink, DataSource, pass_data_internal};
use crate::grid::{GridPosition, GridSprite, Orientation};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::prelude::{Bundle, Commands, Component, Query, Res, SpawnWith, Time};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

#[derive(Component, Clone)]
pub struct Splitter {
    pub(crate) throughput: f32,
    pub(crate) source_count: i64,
}

impl Building for Splitter {
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
                        for i in 0..source_count {
                            spawner.spawn((
                                GridSprite(Color::linear_rgba(0.1, 0.3, 1.0, 0.3)),
                                DataSource {
                                    direction: orientation.effective_direction(),
                                    throughput,
                                    limited: true,
                                    buffer: DataBuffer::default(),
                                },
                                position.offset(orientation.layout_direction(), i as i64),
                            ));
                        }
                        spawner.spawn((
                            DataSink {
                                direction: orientation.direction.opposite(),
                                buffer: DataBuffer::default(),
                            },
                            position,
                            GridSprite(Color::linear_rgba(0.1, 0.3, 1.0, 0.3)),
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
            sprite: SpriteResource::Atlas(self.source_count as usize + 1),
            grid_width: self.source_count,
            grid_height: 1,
            cost: 60,
            name: format!("Splitter {}x1", self.source_count),
            building_type: BuildingTypes::Splitter(Splitter {
                throughput: 5.0,
                source_count: self.source_count,
            }),
        }
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
