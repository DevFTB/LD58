use crate::factory::logical::{DataBuffer, DataSink, DataSource, Dataset};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::children;
use bevy::prelude::SpawnRelated;
use bevy::prelude::{Bundle, Component};

pub mod aggregator;

#[derive(Component)]
pub struct SourceBuilding;
impl SourceBuilding {
    pub fn get_spawn_bundle(
        position: GridPosition,
        direction: Direction,
        packet: Dataset,
    ) -> impl Bundle {
        (
            SourceBuilding,
            position,
            children![(
                DataSource {
                    throughput: 1.,
                    output_direction: direction,
                    buffer: DataBuffer {
                        shape: Some(packet),
                        value: 0.
                    },
                    limited: false,
                },
                position
            )],
            GridSprite(Color::linear_rgba(0., 1., 0., 1.)),
        )
    }
}

#[derive(Component)]
pub struct SinkBuilding;

impl SinkBuilding {
    pub fn get_spawn_bundle(
        position: GridPosition,
        direction: Direction,
        shape: Option<Dataset>,
    ) -> impl Bundle {
        (
            SinkBuilding,
            position,
            children![(
                DataSink {
                    input_direction: direction,
                    buffer: DataBuffer { shape, value: 0. }
                },
                position,
            )],
            GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
        )
    }
}
