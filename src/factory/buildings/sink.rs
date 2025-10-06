use crate::factory::buildings::buildings::{Building, BuildingData};
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink};
use crate::grid::{Direction, GridAtlasSprite, GridPosition, Orientation};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::math::I64Vec2;
use bevy::prelude::{Commands, Component, Entity};
use bevy::prelude::{SpawnRelated, SpawnWith};
use bevy::sprite::Text2d;
use std::ops::Add;

#[derive(Component, Clone)]
pub struct SinkBuilding {
    pub size: I64Vec2,
}

impl Building for SinkBuilding {
    fn spawn_naked(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        let directions = Direction::ALL
            .iter()
            .map(|dir| orientation.transform_relative(*dir))
            .collect::<Vec<_>>();

        let sink_bundles = match (self.size.x, self.size.y) {
            (0, 0) => Vec::new(),
            (1, 1) => directions
                .iter()
                .map(|dir| (*dir, position))
                .collect::<Vec<_>>(),
            (x, y) => {
                let mut indexes: Vec<((i64, i64), TilePlacement)> =
                    Vec::with_capacity((self.size.x * self.size.y) as usize);
                for x in 0..self.size.x {
                    for y in 0..self.size.y {
                        let placement = get_placement((x, y), self.size);
                        indexes.push(((x, y), placement));
                    }
                }
                indexes
                    .iter()
                    .filter_map(|((x, y), placement)| match placement {
                        TilePlacement::Inner => None,
                        TilePlacement::Edge(side) => {
                            let position = GridPosition((*position).add(I64Vec2::new(*x, *y)));
                            Some(vec![(*side, position)])
                        }
                        TilePlacement::Corner(h, v) => {
                            let pos = GridPosition((*position).add(I64Vec2::new(*x, *y)));
                            Some(vec![(*h, pos), (*v, pos)])
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>()
            }
        }
        .iter()
        .map(|(dir, pos)| {
            (
                DataSink {
                    direction: *dir,
                    buffer: DataBuffer {
                        shape: None,
                        value: 0.,
                    },
                },
                *pos,
                GridAtlasSprite {
                    grid_height: 1,
                    grid_width: 1,
                    atlas_index: 1,
                    orientation: Orientation {
                        direction: Direction::Up,
                        flipped: false,
                    },
                },
                Text2d(String::default()),
            )
        })
        .collect::<Vec<_>>();

        commands
            .spawn((
                position,
                Tiles::spawn(SpawnWith(
                    move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                        sink_bundles.into_iter().for_each(|b| {
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
            name: String::from("Sink"),
            cost: 0,
            grid_width: self.size.x,
            grid_height: self.size.y,
            sprite: None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePlacement {
    Inner,
    Edge(Direction),
    Corner(Direction, Direction),
}

fn get_placement((x, y): (i64, i64), size: I64Vec2) -> TilePlacement {
    assert!(size.x > 0 && size.y > 0, "size must be > 0");

    let left = x == 0;
    let right = x == size.x - 1;
    let top = y == 0;
    let bottom = y == size.y - 1;

    let h_edge = left || right;
    let v_edge = top || bottom;

    match (h_edge, v_edge) {
        (true, true) => {
            let h_dir = if left {
                Direction::Left
            } else {
                Direction::Right
            };
            // swapped: y == 0 -> Down, y == size-1 -> Up
            let v_dir = if top { Direction::Down } else { Direction::Up };
            TilePlacement::Corner(h_dir, v_dir)
        }
        (true, false) => {
            let h_dir = if left {
                Direction::Left
            } else {
                Direction::Right
            };
            TilePlacement::Edge(h_dir)
        }
        (false, true) => {
            // swapped: y == 0 -> Down, y == size-1 -> Up
            let v_dir = if top { Direction::Down } else { Direction::Up };
            TilePlacement::Edge(v_dir)
        }
        (false, false) => TilePlacement::Inner,
    }
}
