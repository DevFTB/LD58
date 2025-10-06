use crate::factory::buildings::buildings::{Building, BuildingData, SpriteResource};
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSource, Dataset};
use crate::grid::{Direction, GridPosition, GridSprite, Orientation};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::math::I64Vec2;
use bevy::prelude::{Commands, Component, Entity};
use bevy::prelude::{SpawnRelated, SpawnWith};

#[derive(Component, Clone)]
pub struct SourceBuilding {
    pub(crate) directions: Vec<Direction>,
    pub(crate) throughput: f32,
    pub(crate) limited: bool,
    pub(crate) size: I64Vec2,
    pub(crate) shape: Dataset,
}

impl Building for SourceBuilding {
    fn spawn_naked(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        let throughput_per_side = self.throughput / self.directions.len() as f32;
        let directions = self
            .directions
            .iter()
            .map(|dir| orientation.transform_relative(*dir))
            .collect::<Vec<_>>();
        let shape = self.shape.clone();
        let bundles = directions
            .iter()
            .map(|dir| {
                (
                    DataSource {
                        direction: *dir,
                        throughput: throughput_per_side,
                        buffer: DataBuffer {
                            shape: Some(shape.clone()),
                            value: 0.,
                        },
                        limited: self.limited,
                    },
                    position,
                    GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                    // Text2d removed for performance - thousands of sources × 4 directions = 10k+ text entities
                    // Text2d::new("0"),
                )
            })
            .collect::<Vec<_>>();

        commands
            .spawn((
                position,
                Tiles::spawn(SpawnWith(
                    move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                        bundles.into_iter().for_each(|b| {
                            spawner.spawn(b);
                        });
                    },
                )),
                self.clone(),
            ))
            .id()
    }

    fn data(&self) -> BuildingData {
        BuildingData {
            sprite: Some(SpriteResource::Atlas(1)),
            grid_width: self.size.x,
            grid_height: self.size.y,
            cost: 0,
            name: "Source".to_string(),
        }
    }
}
