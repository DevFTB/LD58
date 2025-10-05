use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSource, Dataset};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::prelude::{Bundle, Component};
use bevy::prelude::{SpawnRelated, SpawnWith};
use bevy::sprite::Text2d;

#[derive(Component)]
pub struct SourceBuilding;

impl SourceBuilding {
    pub fn get_bundle(
        position: GridPosition,
        directions: Vec<Direction>,
        shape: Dataset,
        throughput: f32,
        limited: bool,
    ) -> impl Bundle {
        (
            SourceBuilding,
            position,
            Tiles::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                    directions.iter().for_each(|dir| {
                        spawner.spawn((
                            DataSource {
                                direction: *dir,
                                throughput: throughput / directions.len() as f32,
                                buffer: DataBuffer {
                                    shape: Some(shape.clone()),
                                    value: 0.,
                                },
                                limited,
                            },
                            position,
                            GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                            // Text2d removed for performance - thousands of sources × 4 directions = 10k+ text entities
                            // Text2d::new("0"),
                        ));
                    });
                },
            )),
        )
    }
}
