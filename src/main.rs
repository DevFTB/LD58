extern crate core;

use bevy_prng::WyRand;
use bevy_rand::prelude::*;

use crate::{
    assets::AssetPlugin,
    camera::GameCameraPlugin,
    events::EventsPlugin,
    factions::FactionsPlugin,
    factory::{physical::PhysicalLink, FactoryPlugin},
    grid::{Grid, GridPlugin, GridPosition},
    ui::UIPlugin,
    world_gen::WorldGenPlugin,
    contracts::ContractsPlugin,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

mod assets;
mod camera;
mod events;
mod factions;
mod factory;
mod grid;
// mod test; // TODO: Update test functions with new bundle signatures
mod test;
mod ui;
mod world_gen;
mod contracts;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(AssetPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(EventsPlugin)
        .add_plugins(ContractsPlugin)
        .add_plugins(GameCameraPlugin)
        .add_plugins(WorldGenPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FactoryPlugin)
        .add_plugins(FactionsPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, remove_physical_link_on_right_click)
        .run();
}

fn startup(mut commands: Commands) {
    test::spawn_splitter_test(&mut commands);
    //test::spawn_delinker_test(&mut commands);
    //test::spawn_combiner_test(&mut commands);
    //test::spawn_trunking_test(&mut commands);
    //test::spawn_sized_sink_test(&mut commands);
}

pub fn remove_physical_link_on_right_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    // Use your main 2D camera; if you have a marker component for it, add With<YourCameraTag>
    camera_q: Query<(&Camera, &GlobalTransform)>,
    grid: Res<Grid>,

    links: Query<(Entity, &GridPosition), With<PhysicalLink>>,
) {
    // Only act on the press edge to avoid repeating every frame the button is held.
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let (camera, cam_xform) = match camera_q.single() {
        Ok(c) => c,
        Err(_) => return,
    };
    let cursor_screen = match window.cursor_position() {
        Some(p) => p,
        None => return, // cursor not over window
    };

    // 2D conversion from screen to world
    let world_pos = match camera.viewport_to_world_2d(cam_xform, cursor_screen) {
        Ok(p) => p,
        Err(_) => return,
    };

    // Find a PhysicalLink occupying this grid cell
    if let Some((entity, _)) = links
        .iter()
        .find(|(_, gp)| **gp == grid.world_to_grid(world_pos))
    {
        // Option A: fully despawn the entity (removes sprite, etc.)
        commands.entity(entity).remove::<PhysicalLink>();
        commands.entity(entity).despawn();

        // Option B: only remove the PhysicalLink component (keeps the entity/sprite)
    }
}
