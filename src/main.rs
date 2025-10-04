use bevy::{math::I8Vec2, platform::collections::HashMap, prelude::*};
use bevy::window::PrimaryWindow;
use crate::{
    camera::GameCameraPlugin,
    factory::{
        FactoryPlugin,
        logical::{Dataset, Sink, Source},
        physical::PhysicalLink,
    },
    grid::{Grid, GridPlugin, GridPosition},
};
use crate::grid::Direction;

mod camera;
mod factory;
mod grid;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(GameCameraPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FactoryPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, remove_physical_link_on_right_click)
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
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I8Vec2 {
        x: 3,
        y: 2,
    })));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I8Vec2 {
        x: 3,
        y: 3,
    })));
    commands.spawn(Sink::get_spawn_bundle(
        GridPosition(I8Vec2 { x: 4, y: 2 }),
        Direction::Left,
        Dataset {
            contents: HashMap::new(),
        },
    ));
    commands.spawn(Sink::get_spawn_bundle(
        GridPosition(I8Vec2 { x: 3, y: 4 }),
        Direction::Left,
        Dataset {
            contents: HashMap::new(),
        },
    ));
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
    if let Some((entity, _)) = links.iter().find(|(_, gp)| **gp == grid.world_to_grid(world_pos)) {
        // Option A: fully despawn the entity (removes sprite, etc.)
        commands.entity(entity).remove::<PhysicalLink>();
        commands.entity(entity).despawn();

        // Option B: only remove the PhysicalLink component (keeps the entity/sprite)
    }
}