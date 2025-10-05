use bevy::{math::I8Vec2, platform::collections::HashMap, prelude::*};

use crate::{
    camera::GameCameraPlugin,
    events::EventsPlugin,
    factory::{
        FactoryPlugin,
        logical::{Dataset, Sink, Source},
        physical::PhysicalLink,
    },
    grid::{GridPlugin, GridPosition},
    factions::FactionsPlugin,
    ui::{UIPlugin},
    assets::AssetPlugin
};
use crate::grid::Direction;

mod camera;
mod events;
mod factory;
mod grid;
mod factions;
mod things;
mod ui;
mod assets;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(AssetPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(EventsPlugin)
        .add_plugins(GameCameraPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FactoryPlugin)
        .add_plugins(FactionsPlugin)
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
