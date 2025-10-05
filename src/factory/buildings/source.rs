use crate::factory::buildings::Tiles;
use crate::factory::logical::{DataBuffer, DataSource, Dataset};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::related;
use bevy::prelude::SpawnRelated;
use bevy::prelude::{Bundle, Component};

#[derive(Component)]
pub struct SourceBuilding;

impl SourceBuilding {
    pub fn get_bundle(
        position: GridPosition,
        direction: Direction,
        packet: Dataset,
        throughput: f32,
    ) -> impl Bundle {
        (
            SourceBuilding,
            position,
            related!(
                Tiles[(
                    DataSource {
                        throughput,
                        direction: direction.clone(),
                        buffer: DataBuffer {
                            shape: Some(packet),
                            value: 0.,
                        },
                        limited: false,
                    },
                    GridSprite(Color::linear_rgba(0., 1., 0., 1.)),
                    position.clone(),
                )]
            ),
        )
    }
}
