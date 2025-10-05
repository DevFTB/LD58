use bevy::{math::Vec2, platform::collections::HashMap, prelude::*};
use bevy_rand::prelude::*;
use bevy_prng::WyRand;

use crate::{
    camera::GameCameraPlugin,
    factory::{
        FactoryPlugin,
        logical::{Dataset, Sink, Source},
        physical::PhysicalLink,
    },
    grid::{Grid, GridPlugin, GridPosition},
    ui::{UIPlugin},
    world_gen::{WorldGenPlugin},
};
use crate::grid::Direction;

mod camera;
mod factory;
mod grid;
mod ui;
mod world_gen;

#[derive(Component, Default, Debug, Clone)]
pub enum Faction {
    Government,
    #[default]
    Corporate,
    Academia,
    Criminal
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(GameCameraPlugin)
        .add_plugins(WorldGenPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FactoryPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Source::get_spawn_bundle(
        GridPosition(IVec2 { x: 1, y: 1 }),
        Direction::Right,
        Dataset {
            contents: HashMap::new(),
        }
    ));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(IVec2 {
        x: 2,
        y: 1,
    })));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(IVec2 {
        x: 3,
        y: 1,
    })));
    commands.spawn(Sink::get_spawn_bundle(
        GridPosition(IVec2 { x: 4, y: 1 }),
        Direction::Left,
        Dataset {
            contents: HashMap::new(),
        },
    ));
}
