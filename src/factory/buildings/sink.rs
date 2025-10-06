use crate::contracts::SinkContracts;
use crate::factory::buildings::buildings::{Building, BuildingData};
use crate::factory::buildings::{Tile, Tiles};
use crate::factory::logical::{DataBuffer, DataSink};
use crate::grid::{Direction, GridAtlasSprite, GridPosition, Orientation};
use bevy::ecs::relationship::RelatedSpawner;
use bevy::math::I64Vec2;
use bevy::prelude::{Commands, Component, Entity, Query, Changed, Res};
use bevy::prelude::{SpawnRelated, SpawnWith};
use bevy::sprite::Text2d;
use bevy::text::{TextFont, TextColor};
use bevy::color::Color;
use bevy::transform::components::Transform;
use bevy::time::Time;
use std::collections::VecDeque;
use std::ops::Add;

#[derive(Component, Clone)]
#[require(SinkContracts)]
pub struct SinkBuilding {
    pub size: I64Vec2,
}

/// Component to track moving average of sink throughput over 2 seconds
#[derive(Component)]
pub struct ThroughputTracker {
    /// Stores (timestamp, value) pairs for the last 2 seconds
    pub samples: VecDeque<(f32, f32)>,
    /// Current moving average throughput
    pub average_throughput: f32,
}

impl ThroughputTracker {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            average_throughput: 0.0,
        }
    }
    
    /// Add a new sample and calculate moving average
    pub fn add_sample(&mut self, timestamp: f32, value: f32) {
        self.samples.push_back((timestamp, value));
        
        // Remove samples older than 2 seconds
        let cutoff_time = timestamp - 2.0;
        while let Some(&(sample_time, _)) = self.samples.front() {
            if sample_time < cutoff_time {
                self.samples.pop_front();
            } else {
                break;
            }
        }
        
        // Calculate moving average
        if self.samples.is_empty() {
            self.average_throughput = 0.0;
        } else {
            let sum: f32 = self.samples.iter().map(|(_, value)| value).sum();
            self.average_throughput = sum / self.samples.len() as f32;
        }
    }
}
pub struct SinkThroughput(f32);

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
                    buffer: DataBuffer::default(),
                },
               ThroughputTracker::new(),
                *pos,
                GridAtlasSprite {
                    grid_height: 1,
                    grid_width: 1,
                    atlas_index: 1,
                    orientation,
                },
                Text2d::new(""),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)), // Yellow text
                Transform::from_translation(bevy::math::Vec3::new(0.0, -16.0, 1.0)), // Offset below tile
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

/// System to update debug text on sinks showing their buffer values
pub fn update_sink_debug_text(
    mut query: Query<(&DataSink, &ThroughputTracker, &mut Text2d)>,
) {
    for (sink, tracker, mut text) in query.iter_mut() {
        // Format buffer information: type, value, and throughput
        let buffer_info = format!(
            "{}",
            tracker.average_throughput
        );
        **text = buffer_info;
    }
}

/// System to update sink throughput based on moving averages of last_in over 30 seconds
pub fn update_sink_throughput(
    mut query: Query<(&DataSink, &mut ThroughputTracker)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    for (sink, mut tracker) in query.iter_mut() {
        // Add current buffer last_in as a sample
        tracker.add_sample(current_time, sink.buffer.last_in);
    }
}
