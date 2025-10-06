use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::math::I64Vec2;
use bevy::prelude::{Changed, DerefMut};
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
    },
    math::{Vec2, Vec3, Vec4, primitives::Rectangle},
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
pub struct WorldMap(pub HashMap<GridPosition, Vec<Entity>>);

// Function to check if a set of grid positions is free
#[derive(Component, Deref, PartialEq, Eq, Hash, Copy, Clone, Default)]
#[require(Transform)]
#[component(on_insert = grid_position_added)]
#[component(on_remove = grid_position_removed)]
#[derive(Debug)]
pub struct GridPosition(pub I64Vec2);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Direction {
    Right,
    Down,
    Left,
    Up,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::Up,
    ];
}

/// Represents the orientation of a building (direction + flip state)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Orientation {
    pub direction: Direction,
    pub flipped: bool,
}

impl Orientation {
    pub fn new(direction: Direction, flipped: bool) -> Self {
        Self { direction, flipped }
    }

    /// Calculate the effective direction for occupied cells calculation.
    /// Only flips the direction for Left/Right orientations when flipped.
    /// For Up/Down, the effective direction stays the same (flip is handled by anchor offset).
    pub fn effective_direction(&self) -> Direction {
        self.direction.calculate_effective_direction(self.flipped)
    }

    /// Get the rotation angle in radians for this orientation
    pub fn rotation_angle(&self) -> f32 {
        self.direction.rotation_angle()
    }

    /// Rotate the orientation clockwise
    pub fn rotate_clockwise(&self) -> Self {
        Self {
            direction: self.direction.rotate_clockwise(),
            flipped: self.flipped,
        }
    }

    /// Rotate the orientation counterclockwise
    pub fn rotate_counterclockwise(&self) -> Self {
        Self {
            direction: self.direction.rotate_counterclockwise(),
            flipped: self.flipped,
        }
    }

    /// Toggle the flipped state
    pub fn toggle_flip(&self) -> Self {
        Self {
            direction: self.direction,
            flipped: !self.flipped,
        }
    }

    /// Transform a direction that is relative to the default (Up) orientation
    /// into the world direction for this orientation (taking `flipped` into account).
    pub fn transform_relative(&self, dir: Direction) -> Direction {
        // Step 1: apply flip in local (Up-based) coords: swap Left <-> Right
        let local = if self.flipped {
            match dir {
                Direction::Left => Direction::Right,
                Direction::Right => Direction::Left,
                other => other,
            }
        } else {
            dir
        };

        // Step 2: rotate local (Up-based) direction into world direction
        match self.direction {
            Direction::Up => local,
            Direction::Right => match local {
                Direction::Up => Direction::Right,
                Direction::Right => Direction::Down,
                Direction::Down => Direction::Left,
                Direction::Left => Direction::Up,
            },
            Direction::Down => match local {
                Direction::Up => Direction::Down,
                Direction::Right => Direction::Left,
                Direction::Down => Direction::Up,
                Direction::Left => Direction::Right,
            },
            Direction::Left => match local {
                Direction::Up => Direction::Left,
                Direction::Right => Direction::Up,
                Direction::Down => Direction::Right,
                Direction::Left => Direction::Down,
            },
        }
    }

    /// Get the layout direction - which way the building extends from the anchor.
    /// This represents the perpendicular direction to the facing direction.
    /// This is useful for determining tile placement in multi-tile buildings.
    pub fn layout_direction(&self) -> Direction {
        // For Up/Down, flip changes the layout direction
        // For Left/Right
        match self.direction {
            Direction::Up => {
                if self.flipped {
                    Direction::Right // Flipped: extends right from anchor
                } else {
                    Direction::Left // Normal: extends left from anchor
                }
            }
            Direction::Down => {
                if self.flipped {
                    Direction::Left // Flipped: extends left from anchor
                } else {
                    Direction::Right // Normal: extends right from anchor
                }
            }
            Direction::Right => {
                // Always extends up (counterclockwise from Right)
                Direction::Up
            }
            Direction::Left => {
                // Always extends down (counterclockwise from Left)
                Direction::Down
            }
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            direction: Direction::Up,
            flipped: false,
        }
    }
}

#[derive(Resource)]
pub struct Grid {
    pub scale: f32,
    pub base_offset: f32,
}

#[derive(Component, Deref)]
pub struct GridSprite(pub Color);

/// Component for buildings that use texture atlas sprites
/// Contains the atlas index and size information for proper rendering
#[derive(Component)]
pub struct GridAtlasSprite {
    pub atlas_index: usize,
    pub grid_width: i64,
    pub grid_height: i64,
    pub orientation: Orientation,
}

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
        app.insert_resource(Grid {
            scale: 64.0,
            base_offset: 0.,
        });
        app.insert_resource(WorldMap::default());
        app.add_plugins(Material2dPlugin::<GridMaterial>::default());
        app.add_systems(Startup, setup_grid);
        app.add_systems(
            PostUpdate,
            (
                transform_to_grid,
                spawn_grid_sprite_system,
                spawn_grid_atlas_sprite_system,
            ),
        );
    }
}

impl Grid {
    // Helper: convert a world position to a GridPosition by snapping to the grid.
    pub fn world_to_grid(&self, world: Vec2) -> GridPosition {
        let p = (world - self.base_offset) / self.scale;
        // Use floor for "lower-left origin" style grids; use round() if that's your convention.
        let gx = p.x.floor() as i64;
        let gy = p.y.floor() as i64;
        GridPosition(I64Vec2 { x: gx, y: gy })
    }

    // bottom left corner
    pub fn grid_to_world_corner(&self, pos: &GridPosition) -> Vec2 {
        Vec2::new(
            pos.x as f32 * self.scale + self.base_offset,
            pos.y as f32 * self.scale + self.base_offset,
        )
    }

    pub fn grid_to_world_center(&self, pos: &GridPosition) -> Vec2 {
        Vec2::new(
            pos.x as f32 * self.scale + self.base_offset + self.scale / 2.0,
            pos.y as f32 * self.scale + self.base_offset + self.scale / 2.0,
        )
    }

    /// Calculate the world position for a multi-tile building sprite
    /// accounting for anchor position, orientation, and dimensions.
    ///
    /// - `anchor_pos`: The base grid position (bottom-left tile)
    /// - `width`: Grid width of the building
    /// - `height`: Grid height of the building
    /// - `orientation`: The orientation (direction + flip state) of the building
    pub fn calculate_building_sprite_position(
        &self,
        anchor_pos: &GridPosition,
        width: i64,
        height: i64,
        orientation: Orientation,
    ) -> Vec2 {
        let anchor_center = self.grid_to_world_center(anchor_pos);

        // Calculate sprite position based on anchor and direction
        // The sprite extends from the anchor in the direction it's facing
        // For a 3x1 building, sprite center is (width-1)/2 cells from anchor
        let offset_cells = (width - 1) as f32 * self.scale / 2.0;

        // OFFSET CHANGES: Only for Up/Down orientations
        // For Up/Down: flip changes which side the building extends from (anchor behavior)
        // For Left/Right: offset stays the same, flip is handled by sprite.flip_x
        let (x_offset, y_offset) = match orientation.direction {
            Direction::Up => {
                if orientation.flipped {
                    (offset_cells, 0.0) // Flipped: extends right from anchor
                } else {
                    (-offset_cells, 0.0) // Normal: extends left from anchor
                }
            }
            Direction::Down => {
                if orientation.flipped {
                    (-offset_cells, 0.0) // Flipped: extends left from anchor
                } else {
                    (offset_cells, 0.0) // Normal: extends right from anchor
                }
            }
            Direction::Right => (0.0, offset_cells), // Extends up, flip doesn't change offset
            Direction::Left => (0.0, -offset_cells), // Extends down, flip doesn't change offset
        };

        Vec2::new(anchor_center.x + x_offset, anchor_center.y + y_offset)
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

    pub fn rotate_clockwise(&self) -> Direction {
        match self {
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Up => Direction::Right,
        }
    }

    /// Calculate the effective direction for occupied cells calculation.
    /// Only flips the direction for Left/Right orientations when flipped.
    /// For Up/Down, the effective direction stays the same (flip is handled by anchor offset).
    pub fn calculate_effective_direction(&self, flipped: bool) -> Direction {
        if flipped && matches!(self, Direction::Left | Direction::Right) {
            self.opposite()
        } else {
            *self
        }
    }

    /// Get the rotation angle in radians for this direction
    /// Up = 0, Right = -90°, Down = -180°, Left = -270° (or 90°)
    pub fn rotation_angle(&self) -> f32 {
        use std::f32::consts::{FRAC_PI_2, PI};
        match self {
            Direction::Up => 0.0,
            Direction::Right => -FRAC_PI_2,
            Direction::Down => -PI,
            Direction::Left => FRAC_PI_2, // -270° same as 90°
        }
    }

    pub fn rotate_counterclockwise(&self) -> Direction {
        match self {
            Direction::Right => Direction::Up,
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
        }
    }
}

impl GridPosition {
    pub fn neighbours(&self) -> Vec<(Direction, GridPosition)> {
        vec![
            (
                Direction::Left,
                GridPosition(I64Vec2 {
                    x: self.x - 1,
                    y: self.y,
                }),
            ),
            (
                Direction::Right,
                GridPosition(I64Vec2 {
                    x: self.x + 1,
                    y: self.y,
                }),
            ),
            (
                Direction::Up,
                GridPosition(I64Vec2 {
                    x: self.x,
                    y: self.y + 1,
                }),
            ),
            (
                Direction::Down,
                GridPosition(I64Vec2 {
                    x: self.x,
                    y: self.y - 1,
                }),
            ),
        ]
    }

    /// Returns a new GridPosition offset by one tile in the given direction.
    pub fn offset(&self, direction: Direction, amount: i64) -> GridPosition {
        match direction {
            Direction::Right => GridPosition(I64Vec2::new(self.0.x + amount, self.0.y)),
            Direction::Up => GridPosition(I64Vec2::new(self.0.x, self.0.y + amount)),
            Direction::Left => GridPosition(I64Vec2::new(self.0.x - amount, self.0.y)),
            Direction::Down => GridPosition(I64Vec2::new(self.0.x, self.0.y - amount)),
        }
    }
}
impl From<I64Vec2> for GridPosition {
    fn from(value: I64Vec2) -> Self {
        GridPosition(value)
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

    world_map
        .entry(grid_position)
        .or_insert_with(Vec::new)
        .push(entity);
}

fn grid_position_removed(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    let grid_position = world.get::<GridPosition>(entity).unwrap().clone();
    let mut world_map = world.get_resource_mut::<WorldMap>().unwrap();

    if let Some(entities) = world_map.get_mut(&grid_position) {
        entities.retain(|&e| e != entity);
        // Remove the entry if no entities remain at this position
        if entities.is_empty() {
            world_map.remove(&grid_position);
        }
    }
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
            offset: Vec2::splat(grid.scale / 4.0),
            resolution: Vec2::new(width, height), // Match your quad size
            grid_intensity: 0.7,
        })),
        Transform::from_translation(Vec3 {
            x: grid.base_offset + grid.scale / 2.0,
            y: grid.base_offset + grid.scale / 2.0,
            z: 1.,
        }),
    ));
}

fn transform_to_grid(
    query: Query<(&mut Transform, &GridPosition), Changed<GridPosition>>,
    grid: Res<Grid>,
) {
    for (mut transform, grid_pos) in query {
        let vec2 = grid.grid_to_world_center(grid_pos);
        transform.translation = Vec3::new(vec2.x, vec2.y, transform.translation.z);
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

/// System to spawn texture atlas sprites for buildings on the grid
/// This handles multi-tile buildings by calculating the proper size and position
fn spawn_grid_atlas_sprite_system(
    mut commands: Commands,
    grid: Res<Grid>,
    game_assets: Res<crate::assets::GameAssets>,
    query: Query<(Entity, &GridAtlasSprite, &GridPosition), Added<GridAtlasSprite>>,
) {
    use bevy::prelude::TextureAtlas;

    for (entity, atlas_sprite, grid_pos) in &query {
        // Calculate the sprite size in pixels based on grid dimensions
        let sprite_width = atlas_sprite.grid_width as f32 * grid.scale;
        let sprite_height = atlas_sprite.grid_height as f32 * grid.scale;

        // Use the shared anchoring function to calculate proper position
        let position = grid.calculate_building_sprite_position(
            grid_pos,
            atlas_sprite.grid_width,
            atlas_sprite.grid_height,
            atlas_sprite.orientation,
        );

        // Calculate rotation angle based on orientation
        let rotation_angle = atlas_sprite.orientation.rotation_angle();

        commands.entity(entity).insert((
            Sprite {
                custom_size: Some(Vec2::new(sprite_width, sprite_height)),
                image: game_assets.machines_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: game_assets.machines_layout.clone(),
                    index: atlas_sprite.atlas_index,
                }),
                flip_x: atlas_sprite.orientation.flipped, // Always apply flip_x when flipped
                ..Default::default()
            },
            Transform::from_translation(position.extend(0.0))
                .with_rotation(bevy::prelude::Quat::from_rotation_z(rotation_angle)),
        ));
    }
}
pub fn calculate_occupied_cells(base_position: I64Vec2, width: i64, height: i64) -> Vec<I64Vec2> {
    let mut cells = Vec::new();
    for dx in 0..width {
        for dy in 0..height {
            cells.push(I64Vec2::new(base_position.x + dx, base_position.y + dy));
        }
    }
    cells
}

pub fn calculate_occupied_cells_rotated(
    anchor_position: I64Vec2,
    width: i64,
    height: i64,
    orientation: Orientation,
) -> Vec<I64Vec2> {
    let mut cells = Vec::new();

    // Get the layout direction (which way the building extends from anchor)
    let layout_dir = orientation.layout_direction();

    // Building extends in the layout direction from the anchor
    for i in 0..width {
        let offset = match layout_dir {
            Direction::Up => I64Vec2::new(0, i),
            Direction::Down => I64Vec2::new(0, -i),
            Direction::Right => I64Vec2::new(i, 0),
            Direction::Left => I64Vec2::new(-i, 0),
        };
        cells.push(anchor_position + offset);
    }

    cells
}
pub fn are_positions_free(world_map: &WorldMap, positions: &[GridPosition]) -> bool {
    positions.iter().all(|pos| !world_map.0.contains_key(pos))
}
