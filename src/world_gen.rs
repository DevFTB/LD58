use std::collections::{VecDeque};

use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, BROWN, GRAY}, ecs::error::info, math::I64Vec2, platform::collections::{HashMap, HashSet}, prelude::*
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
const WORLD_SIZE: i32 = 500;
const WORLD_MIN: i32 = -(WORLD_SIZE / 2);
const WORLD_MAX: i32 = (WORLD_SIZE / 2) - 1; 

const FACTION_CLUSTER_THRESHOLD: f32 = 0.35;


impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // apply logic to determine which ones start locked
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

    // label faction clusters using a bfs
    let mut locked_queue: Vec<IVec2> = locked_cells.clone();
    let valid_nodes: HashSet<IVec2> = locked_cells.iter().cloned().collect();
    let mut visited: HashSet<IVec2> = HashSet::new();

    // grid_pos -> cluster_id
    let mut cluster_map: HashMap<IVec2, i64> = HashMap::new();

    // cluster_id -> grid_pos
    let mut center_map: HashMap<i64, IVec2> = HashMap::new();

    let mut cluster_id: i64 = 0;

    while let Some(current) = locked_queue.pop() {
        if visited.insert(current) {
            // new cluster
            let mut queue: VecDeque<IVec2> = VecDeque::new();
            let mut cluster_nodes: Vec<IVec2> = Vec::new();

            // TODO: consider sticking noise calc in hashmap at the start if performance issues from recalcs
            let mut center_node = (current, get_locked_tile_noise(current, noise_offset));
            queue.push_back(current);

            while let Some(current_inner) = queue.pop_front() {
                cluster_nodes.push(current_inner);
                let tile_noise = get_locked_tile_noise(current_inner, noise_offset);
                if tile_noise < center_node.1 {
                    center_node = (current_inner, tile_noise);
                }

                for (_, neighbour) in GridPosition::neighbours(&GridPosition(current_inner)) {
                    let neighbour_vec = neighbour.0;
                    if valid_nodes.contains(&neighbour.0) && visited.insert(neighbour_vec) {
                        queue.push_back(neighbour_vec);
                    }
                }
            }

            // println!("cluster nodes: {:?}", cluster_nodes);

            // found all nodes for current cluster: log cluster id for all nodes and center
            cluster_map.extend(
                cluster_nodes.into_iter().map(|key| (key, cluster_id))
            );

            center_map.insert(cluster_id, center_node.0);

            cluster_id += 1;
        } else {
            // cluster already visited
            continue
        }
    }

    // println!("cluster map: {:?}", cluster_map);
    // println!("center map: {:?}", center_map);
    
    // debug printing to ensure that gen logic is working

    for cell_vec in locked_cells {
        commands.spawn((
            Locked,
            GridPosition(cell_vec),
            GridSprite(Color::linear_rgba(0., 0.5, 1., 1.)),
        ));
    }

    for (cell_vec, cluster_id) in cluster_map {
        commands.spawn((
            GridPosition(cell_vec),
            Text2d::new(format!("{cluster_id}")),
        ));
    }

    // map each cluster to a faction
    let cluster_faction: HashMap<i64, Faction> = HashMap::from(center_map.iter()
        .map(|(&cluster_id, center_vec)| {
            (cluster_id, map_grid_pos_to_faction(*center_vec))
        }).collect::<HashMap<i64, Faction>>());

    // debug printing to ensure that gen logic is working
    for (cluster_id, cell_vec) in center_map {
        if let Some(faction) = cluster_faction.get(&cluster_id) {
            commands.spawn((
                GridPosition(cell_vec),
                GridSprite(Color::linear_rgba(1., 0.5, 1., 1.)),
                Text2d::new(format!("{:?}: {cluster_id}", faction)),
                ZIndex(4)
            ));
        } else {
            !panic!("{cluster_id} has no faction");
        }

    }
}

fn map_grid_pos_to_faction(vec: IVec2) -> Faction {
    let y = vec.y;
    let x = vec.x;
    return match ( y >= x, y >= -x ) {
        // top
        (true, true) => Faction::Government,
        // right
        (false, true) => Faction::Corporate,
        // bottom
        (false, false) => Faction::Criminal,
        // left
        (true, false) => Faction::Academia
    };
}

fn get_locked_tile_noise(vec: IVec2, offset: f32) -> f32 {
    const SIMPLEX_FREQUENCY: f32 = 0.035;
    const BIAS_EXPONENT: f32 = 2.0;
    let normalised_simplex_noise = (
        fbm_simplex_2d_seeded(vec.as_vec2() * SIMPLEX_FREQUENCY,
        2,
        2.,
        0.1, 
        48.)
    + 1.0) / 2.0;

    const FREQUENCY: f32 = 0.035;
    return worley_2d((vec.as_vec2() + Vec2::new(offset, offset)) * FREQUENCY, 0.5).x
        + (0.2 * normalised_simplex_noise.powf(BIAS_EXPONENT));
}