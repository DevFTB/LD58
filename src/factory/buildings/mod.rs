use crate::factory::logical::{DataBuffer, DataSink, DataSource, Dataset};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::related;
use bevy::prelude::{Bundle, Component, Deref, DerefMut};
use bevy::prelude::{Entity, SpawnRelated};
use bevy::sprite::Text2d;

pub mod aggregator;
pub mod buildings;
pub(crate) mod combiner;
pub mod delinker;
pub(crate) mod splitter;
pub(crate) mod trunker;

#[derive(Component, Debug, Deref, DerefMut)]
#[relationship_target(relationship = Tile, linked_spawn)]
pub struct Tiles(Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = Tiles)]
pub struct Tile(Entity);

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

#[derive(Component)]
pub struct SinkBuilding;

impl SinkBuilding {
    pub fn get_bundle(
        position: GridPosition,
        direction: Direction,
        shape: Option<Dataset>,
    ) -> impl Bundle {
        (
            SinkBuilding,
            position,
            related!(
                Tiles[(
                    DataSink {
                        direction,
                        buffer: DataBuffer { shape, value: 0. }
                    },
                    position,
                    GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                    Text2d::new("0"),
                )]
            ),
        )
    }
}
