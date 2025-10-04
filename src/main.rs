use bevy::{math::I8Vec2, platform::collections::HashMap, prelude::*};

use crate::{
    camera::GameCameraPlugin,
    factory::{
        FactoryPlugin,
        logical::{Dataset, Sink, Source},
        physical::PhysicalLink,
    },
    grid::{Grid, GridPlugin, GridPosition},
    ui::{UIPlugin}
};
use crate::grid::Direction;

mod camera;
mod factory;
mod grid;
mod things;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(GameCameraPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FactoryPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Source::get_spawn_bundle(
        GridPosition(I8Vec2 { x: 1, y: 1 }),
        Direction::Right,
        Dataset {
            contents: HashMap::new(),
        }
    ));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I8Vec2 {
        x: 2,
        y: 1,
    })));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I8Vec2 {
        x: 3,
        y: 1,
    })));
    commands.spawn(Sink::get_spawn_bundle(
        GridPosition(I8Vec2 { x: 4, y: 1 }),
        Direction::Left,
        Dataset {
            contents: HashMap::new(),
        },
    ));
}
