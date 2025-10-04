use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, BROWN, GRAY}, ecs::error::info, prelude::*
};
use noisy_bevy::{fbm_simplex_2d_seeded, worley_2d};
use rand::Rng;
use super::Faction;
use super::grid::{GridPosition, GridSprite};

pub struct WorldGenPlugin;

#[derive(Component)]
pub struct Locked;

#[derive(Component, Default)]
#[require(Transform, GridPosition)]
pub struct Cell;

#[derive(Component)]
#[require(Faction, Cell)]
pub struct FactionSquare{
    cluster_id: i64
}

#[derive(Component)]
pub struct FactionCluster {
    id: i64,
    center: IVec2,
    faction: Faction
}

// might need to change min/max logic a bit if not even lol
const WORLD_SIZE: i32 = 1000;
const WORLD_MIN: i32 = -(WORLD_SIZE / 2);
const WORLD_MAX: i32 = (WORLD_SIZE / 2) - 1; 

// const FACTION_CLUSTER_THRESHOLD: f32 = 0.65;
const FACTION_CLUSTER_THRESHOLD: f32 = 0.15;


impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // apply logic to determine which ones 
    let mut unlocked_cells: Vec<IVec2> = Vec::new();
    let mut locked_cells: Vec<IVec2> = Vec::new();

    let mut rng = rand::rng();
    let noise_offset: f32 = rng.random_range(-1000.0..1000.0);

    for i in WORLD_MIN..=WORLD_MAX {
        for j in WORLD_MIN..=WORLD_MAX {
            let cell_vec = IVec2::new(i, j);
            if get_locked_tile_noise(cell_vec, noise_offset) > FACTION_CLUSTER_THRESHOLD {
                unlocked_cells.push(cell_vec);
            } else {
                locked_cells.push(cell_vec);
            }
        }
    }

    for cell_vec in locked_cells {
        commands.spawn((
            Locked,
            GridPosition(cell_vec),
            GridSprite(Color::linear_rgba(0., 0.5, 1., 1.)),
        ));
    }
}

// fn get_locked_tile_noise(vec: IVec2) -> f32{
//     const SIMPLEX_FREQUENCY: f32 = 0.035;
//     const BIAS_EXPONENT: f32 = 2.0;
//     let normalised =  (fbm_simplex_2d_seeded(vec.as_vec2() * SIMPLEX_FREQUENCY, 2, 2., 0.1, 48.) + 1.0) / 2.0;
//     return normalised.powf(BIAS_EXPONENT);
// }

fn get_locked_tile_noise(vec: IVec2, offset: f32) -> f32 {
    const FREQUENCY: f32 = 0.04;
    return worley_2d((vec.as_vec2() + Vec2::new(offset, offset)) * FREQUENCY, 0.7).x;
}