use crate::factory::buildings::aggregator::Aggregator;
use crate::factory::buildings::splitter::Splitter;
use crate::factory::buildings::{SinkBuilding, SourceBuilding};
use crate::factory::logical::{BasicDataType, DataAttribute, Dataset};
use crate::grid::Direction;
use crate::{
    camera::GameCameraPlugin,
    events::EventsPlugin,
    factory::{physical::PhysicalLink, FactoryPlugin},
    grid::{Grid, GridPlugin, GridPosition},
    factions::FactionsPlugin,
    ui::UIPlugin,
    assets::AssetPlugin
};
use bevy::platform::collections::HashSet;
use bevy::window::PrimaryWindow;
use bevy::{math::I64Vec2, platform::collections::HashMap, prelude::*};


mod factions;
mod assets;
mod camera;
mod events;
mod factory;
mod grid;
mod ui;

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
        .add_systems(Update, remove_physical_link_on_right_click)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(SourceBuilding::get_spawn_bundle(
        GridPosition(I64Vec2 { x: 0, y: 1 }),
        Direction::Right,
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
    ));
    commands.spawn(Aggregator::get_bundle(
        GridPosition(I64Vec2 { x: 1, y: 1 }),
        1.0,
        Direction::Right,
    ));
    // commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I64Vec2 {
    //     x: 1,
    //     y: 1,
    // })));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I64Vec2 {
        x: 2,
        y: 1,
    })));
    commands.spawn(Splitter::get_bundle(
        GridPosition(I64Vec2 { x: 3, y: 1 }),
        50.,
        Direction::Right, /* f32 */ /* grid::Direction */
    ));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I64Vec2 {
        x: 4,
        y: 1,
    })));
    commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I64Vec2 {
        x: 4,
        y: 2,
    })));
    commands.spawn(SinkBuilding::get_spawn_bundle(
        GridPosition(I64Vec2 { x: 5, y: 1 }),
        Direction::Left,
        None,
    ));
    commands.spawn(SinkBuilding::get_spawn_bundle(
        GridPosition(I64Vec2 { x: 5, y: 2 }),
        Direction::Left,
        None,
    ));
    // commands.spawn(PhysicalLink::get_spawn_bundle(GridPosition(I64Vec2 {
    //     x: 3,
    //     y: 3,
    // })));
    // commands.spawn(SinkBuilding::get_spawn_bundle(
    //     GridPosition(I64Vec2 { x: 4, y: 2 }),
    //     Direction::Left,
    //     None,
    // ));
    // commands.spawn(SinkBuilding::get_spawn_bundle(
    //     GridPosition(I64Vec2 { x: 3, y: 4 }),
    //     Direction::Left,
    //     None,
    // ));
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
