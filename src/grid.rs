use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::DerefMut;
use bevy::{
    app::{Plugin, PostUpdate, Startup},
    asset::{Asset, Assets},
    color::Color,
    ecs::{
        component::Component,
        entity::Entity,
        query::Added,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
        world::Ref,
    },
    math::{primitives::Rectangle, I8Vec2, Vec2, Vec3, Vec4},
    mesh::{Mesh, Mesh2d},
    platform::collections::HashMap,
    prelude::Deref,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite::Sprite,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
    transform::components::Transform,
    window::Window,
};

const GRID_SHADER_ASSET_PATH: &str = "shaders/grid_shader.wgsl";
pub struct GridPlugin;

// World map resource to track which grid positions are occupied by which entities
#[derive(Resource, Default, Deref, DerefMut)]
pub struct WorldMap(pub HashMap<GridPosition, Entity>);

// Function to check if a set of grid positions is free
#[derive(Component, Deref, PartialEq, Eq, Hash, Copy, Clone)]
#[require(Transform)]
#[component(on_insert = grid_position_added)]
#[component(on_remove = grid_position_removed)]
pub struct GridPosition(pub I8Vec2);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Direction {
    Right,
    Down,
    Left,
    Up,
}

#[derive(Resource)]
pub struct Grid {
    pub scale: f32,
}

#[derive(Component, Deref)]
pub struct GridSprite(pub Color);

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct GridMaterial {
    #[uniform(0)]
    pub line_colour: Vec4,
    #[uniform(0)]
    pub line_width: f32,
    #[uniform(0)]
    pub grid_size: f32,
    #[uniform(0)]
    pub offset: Vec2,
    #[uniform(0)]
    pub resolution: Vec2,
    #[uniform(0)]
    pub grid_intensity: f32,
}

impl Plugin for GridPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(Grid { scale: 64.0 });
        app.insert_resource(WorldMap::default());
        app.add_plugins(Material2dPlugin::<GridMaterial>::default());
        app.add_systems(Startup, setup_grid);
        app.add_systems(PostUpdate, (transform_to_grid, spawn_grid_sprite_system));
    }
}

impl Grid {
    // Helper: convert a world position to a GridPosition by snapping to the grid.
    pub fn world_to_grid(&self, world: Vec2) -> GridPosition {
        let p = (world + self.scale / 2.) / self.scale;
        // Use floor for "lower-left origin" style grids; use round() if that's your convention.
        let gx = p.x.floor() as i8;
        let gy = p.y.floor() as i8;
        GridPosition(I8Vec2 { x: gx, y: gy })
    }
}
impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Down,
        }
    }
}

impl GridPosition {
    pub fn neighbours(&self) -> Vec<(Direction, GridPosition)> {
        vec![
            (
                Direction::Left,
                GridPosition(I8Vec2 {
                    x: self.x - 1,
                    y: self.y,
                }),
            ),
            (
                Direction::Right,
                GridPosition(I8Vec2 {
                    x: self.x + 1,
                    y: self.y,
                }),
            ),
            (
                Direction::Up,
                GridPosition(I8Vec2 {
                    x: self.x,
                    y: self.y - 1,
                }),
            ),
            (
                Direction::Down,
                GridPosition(I8Vec2 {
                    x: self.x,
                    y: self.y + 1,
                }),
            ),
        ]
    }

    /// Returns a new GridPosition offset by one tile in the given direction.
    pub fn add(&self, direction: Direction) -> GridPosition {
        match direction {
            Direction::Right => GridPosition(I8Vec2::new(self.0.x + 1, self.0.y)),
            Direction::Down => GridPosition(I8Vec2::new(self.0.x, self.0.y + 1)),
            Direction::Left => GridPosition(I8Vec2::new(self.0.x - 1, self.0.y)),
            Direction::Up => GridPosition(I8Vec2::new(self.0.x, self.0.y - 1)),
        }
    }
}

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        GRID_SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
fn grid_position_added(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    let grid_position = world.get::<GridPosition>(entity).unwrap().clone();
    let mut world_map = world.get_resource_mut::<WorldMap>().unwrap();

    world_map.insert(grid_position, entity);
}

fn grid_position_removed(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    let grid_position = world.get::<GridPosition>(entity).unwrap().clone();
    let mut world_map = world.get_resource_mut::<WorldMap>().unwrap();

    world_map.remove(&grid_position);
}

fn setup_grid(
    mut commands: Commands,
    query: Query<&Window>,
    grid: Res<Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GridMaterial>>,
) {
    let window = query.single().unwrap();

    let width = window.width() * 100.;
    let height = window.height() * 100.;

    // quad
    commands.spawn((
        Mesh2d(meshes.add(Rectangle {
            half_size: Vec2 {
                x: width,
                y: height,
            },
        })),
        MeshMaterial2d(materials.add(GridMaterial {
            line_colour: Vec4::new(1.0, 1.0, 1.0, 0.1),
            line_width: 0.5,
            grid_size: grid.scale / 2.0,
            offset: Vec2::ZERO,
            resolution: Vec2::new(width, height), // Match your quad size
            grid_intensity: 0.7,
        })),
        Transform::from_translation(Vec3 {
            x: grid.scale / 2.,
            y: grid.scale / 2.,
            z: 1.,
        }),
    ));
}

fn transform_to_grid(query: Query<(&mut Transform, Ref<GridPosition>)>, grid: Res<Grid>) {
    for (mut transform, grid_pos) in query {
        transform.translation = Vec3::new(
            grid_pos.x as f32 * grid.scale,
            grid_pos.y as f32 * grid.scale,
            transform.translation.z,
        );
    }
}

/// This system queries for any entity that has a `GridSprite` component
/// added to it in the current frame.
fn spawn_grid_sprite_system(
    mut commands: Commands,
    grid: Res<Grid>,
    // The `Added<GridSprite>` filter is the key to making this work.
    // It makes the query only match entities that just received the component.
    query: Query<(Entity, &GridSprite), Added<GridSprite>>,
) {
    // Iterate over all entities that just got a `GridSprite` component
    for (entity, grid_sprite) in &query {
        // Use `commands.entity(entity)` to add more components to the entity that
        // triggered the system.
        commands.entity(entity).insert(
            // We insert a complete `SpriteBundle` to ensure the entity is renderable.
            // `insert` will add or replace existing components.
            Sprite {
                // Set the sprite's size to match the grid tile size.
                custom_size: Some(Vec2::splat(grid.scale)),
                color: **grid_sprite,
                ..Default::default()
            },
        );
    }
}
pub fn calculate_occupied_cells(base_position: I8Vec2, width: i8, height: i8) -> Vec<I8Vec2> {
    let mut cells = Vec::new();
    for dx in 0..width {
        for dy in 0..height {
            cells.push(I8Vec2::new(base_position.x + dx, base_position.y + dy));
        }
    }
    cells
}
pub fn are_positions_free(world_map: &WorldMap, positions: &[GridPosition]) -> bool {
    positions.iter().all(|pos| !world_map.0.contains_key(pos))
}
