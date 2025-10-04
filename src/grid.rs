use bevy::{
    app::{Plugin, PostUpdate, Startup},
    asset::{Asset, Assets},
    ecs::{
        component::Component,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
        world::Ref,
    },
    math::{I8Vec2, Vec2, Vec3, Vec4, primitives::Rectangle},
    mesh::{Mesh, Mesh2d},
    prelude::Deref,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin, MeshMaterial2d},
    transform::components::Transform,
    window::Window,
};

pub struct GridPlugin;

#[derive(Component, Deref)]
#[require(Transform)]
pub struct GridPosition(pub I8Vec2);

#[derive(Resource)]
pub struct Grid {
    pub scale: f32,
}

const GRID_SHADER_ASSET_PATH: &str = "shaders/grid_shader.wgsl";

impl Plugin for GridPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(Grid { scale: 64.0 });
        app.add_plugins(Material2dPlugin::<GridMaterial>::default());
        app.add_systems(Startup, setup_grid);
        app.add_systems(PostUpdate, (transform_to_grid, draw));
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

fn draw() {}

fn transform_to_grid(query: Query<(&mut Transform, Ref<GridPosition>)>, grid: Res<Grid>) {
    for (mut transform, grid_pos) in query {
        transform.translation = Vec3::new(
            grid_pos.x as f32 * grid.scale,
            grid_pos.y as f32 * grid.scale,
            transform.translation.z,
        );
    }
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

impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        GRID_SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
