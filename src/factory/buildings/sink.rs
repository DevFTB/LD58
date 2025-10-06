use crate::contracts::SinkContracts;
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink, Dataset};
use crate::grid::{Direction, GridPosition, GridSprite};
use bevy::color::Color;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::math::I64Vec2;
use bevy::prelude::{Bundle, Component, Text2d};
use bevy::prelude::{SpawnRelated, SpawnWith};
use std::ops::Add;

#[derive(Component)]
#[require(SinkContracts)]
pub struct SinkBuilding;

impl SinkBuilding {
    pub fn get_bundle(
        position: GridPosition,
        directions: Vec<Direction>,
        shape: Option<Dataset>,
    ) -> impl Bundle {
        (
            SinkBuilding,
            Tiles::spawn(SpawnWith(
                move |spawner: &mut RelatedSpawner<Tile> /* Type */| {
                    directions.iter().for_each(|dir| {
                        spawner.spawn((
                            DataSink {
                                direction: *dir,
                                buffer: DataBuffer {
                                    shape: shape.clone(),
                                    value: 0.,
                                },
                            },
                            position,
                            GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                            Text2d::new("0"),
                        ));
                    });
                },
            )),
        )
    }

    pub fn get_sized_bundle(
        base_position: GridPosition,
        size: i64,
        shape: Option<Dataset>,
    ) -> impl Bundle {
        // build cartesian product of coords
        let mut indexes: Vec<((i64, i64), TilePlacement)> =
            Vec::with_capacity((size * size) as usize);
        for x in 0..size {
            for y in 0..size {
                let placement = get_placement((x, y), size);
                indexes.push(((x, y), placement));
            }
        }
        (
            SinkBuilding,
            Tiles::spawn((SpawnWith(move |spawner: &mut RelatedSpawner<Tile>| {
                indexes
                    .iter()
                    .for_each(|((x, y), placement)| match placement {
                        TilePlacement::Inner => {
                            // println!("Spawning inner {x},{y}");
                            spawner.spawn((
                                GridPosition((*base_position).add(I64Vec2::new(*x, *y))),
                                GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                            ));
                        }
                        TilePlacement::Edge(side) => {
                            let position = GridPosition((*base_position).add(I64Vec2::new(*x, *y)));
                            let shape1 = shape.clone();
                            spawner.spawn((
                                DataSink {
                                    direction: *side,
                                    buffer: DataBuffer {
                                        shape: shape1.clone(),
                                        value: 0.,
                                    },
                                },
                                position,
                                GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                                // Text2d removed for performance
                                // Text2d::new("0"),
                            ));
                        }
                        TilePlacement::Corner(h, v) => {
                            let pos = GridPosition((*base_position).add(I64Vec2::new(*x, *y)));
                            // println!("Corner {:?} {:?} {:?}", pos, *h, *v);
                            spawner.spawn((
                                DataSink {
                                    direction: *h,
                                    buffer: DataBuffer {
                                        shape: shape.clone(),
                                        value: 0.,
                                    },
                                },
                                pos,
                                GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                                // Text2d removed for performance
                                // Text2d::new("0"),
                            ));
                            spawner.spawn((
                                DataSink {
                                    direction: *v,
                                    buffer: DataBuffer {
                                        shape: shape.clone(),
                                        value: 0.,
                                    },
                                },
                                pos,
                                GridSprite(Color::linear_rgba(1.0, 0.0, 0.0, 1.0)),
                                // Text2d removed for performance
                                // Text2d::new("0"),
                            ));
                        }
                    });
            }),)),
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePlacement {
    Inner,
    Edge(Direction),
    Corner(Direction, Direction),
}

fn get_placement((x, y): (i64, i64), size: i64) -> TilePlacement {
    assert!(size > 0, "size must be > 0");

    let left = x == 0;
    let right = x == size - 1;
    let top = y == 0;
    let bottom = y == size - 1;

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
