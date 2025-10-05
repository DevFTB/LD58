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
#[require(FactionComponent, Cell, ClusterID)]
pub struct FactionSquare;

#[derive(Component)]
#[require(ClusterID, FactionComponent)]
pub struct FactionCluster {
    center: IVec2,
}

#[derive(Component, Default)]
#[require(FactionComponent)]
pub struct ClusterID(i64);

// single component abstraction for sinks. these manage contract, hold faction etc.
// can be made up of many sink block children
#[derive(Component)]
pub struct SinkParent;

#[derive(Component, Default)]
pub struct FactionComponent(Faction);

// might need to change min/max logic a bit if not even lol
const WORLD_SIZE: i32 = 500;
const WORLD_MIN: i32 = -(WORLD_SIZE / 2);
const WORLD_MAX: i32 = (WORLD_SIZE / 2) - 1;

const STARTING_AREA_SIZE: i32 = 20;
const INITIAL_FACTION_SINKS: [(IVec2, Faction); 4] = [
    (IVec2::new(0,7), Faction::Government),
    (IVec2::new(7,0), Faction::Corporate),
    (IVec2::new(0,-7), Faction::Criminal),
    (IVec2::new(-7,0), Faction::Academia),
];

const FACTION_CLUSTER_THRESHOLD: f32 = 0.32;


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
            if in_start_area(cell_vec) || get_locked_tile_noise(cell_vec, noise_offset) > FACTION_CLUSTER_THRESHOLD {
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
            GridSprite(Color::linear_rgba(0., 0.5, 1., 0.8)),
        ));
    }

    for (cell_vec, cluster_id) in &cluster_map {
        commands.spawn((
            GridPosition(*cell_vec),
            Text2d::new(format!("{cluster_id}")),
        ));
    }

    // map each cluster to a faction
    let cluster_faction: HashMap<i64, Faction> = HashMap::from(center_map.iter()
        .map(|(&cluster_id, center_vec)| {
            (cluster_id, map_grid_pos_to_faction(*center_vec))
        }).collect::<HashMap<i64, Faction>>());

    // debug printing to ensure that gen logic is working
    for (cluster_id, cell_vec) in &center_map {
        if let Some(faction) = cluster_faction.get(cluster_id) {
            commands.spawn((
                GridPosition(*cell_vec),
                GridSprite(Color::linear_rgba(1., 0.5, 1., 1.)),
                Text2d::new(format!("{:?}: {cluster_id}", faction)),
                ZIndex(4)
            ));
        } else {
            panic!("{cluster_id} has no faction");
        }

    }

    // spawn faction sinks
    for (cluster_id, cell_vec) in &center_map {
        if let Some(faction) = cluster_faction.get(cluster_id) {
            // spawn_faction_sink(*cell_vec, *cluster_id, faction.clone(), &cluster_map, &mut commands);
            spawn_faction_sink(*cell_vec, faction.clone(), Some(&cluster_map), &mut commands);
        } else {
            panic!("{cluster_id} has no faction");
        }
    }

    // spawn intitial faction sinks
    for (cell_vec, faction) in INITIAL_FACTION_SINKS {
        spawn_faction_sink(cell_vec, faction, Option::None, &mut commands);
    }

}

fn in_start_area(vec: IVec2) -> bool {
    return vec.length_squared() < STARTING_AREA_SIZE.pow(2);
}

// this option implementation is sus and hack refactor later lol
fn spawn_faction_sink(vec: IVec2, faction: Faction, cluster_map: Option<&HashMap<IVec2, i64>>, commands: &mut Commands) {
    let mut sink_parent = commands.spawn((
        SinkParent,
        // ClusterID(cluster_id),
        FactionComponent(faction)
    ));

    let mut sink_vecs: Vec::<IVec2> = Vec::new();
    for x in vec.x-1..=vec.x+1 {
        for y in vec.y-1..=vec.y+1 {
            let cur_vec = IVec2::new(x, y);
            if let Some(cluster_map_val) = cluster_map  &&
                cluster_map_val.contains_key(&cur_vec) {
                sink_vecs.push(cur_vec);
            } else {
                sink_vecs.push(cur_vec);
            }
        }
    }

    for grid_vec in sink_vecs {
        commands.spawn((
                GridPosition(grid_vec),
                GridSprite(Color::linear_rgba(1., 1., 1., 1.)),
                ZIndex(3)
        ));
    }  

    // println!("sink_vecs: {:?}", sink_vecs);

    // TODO: figure out how to actually spawn individual sink cells and ultimate hierachy

    // sink_parent.with_children(|parent| {
    //     // todo spawn actual sink stuff lol
    //     for grid_vec in sink_vecs {
    //         parent.spawn((
    //                 GridPosition(grid_vec),
    //                 GridSprite(Color::linear_rgba(1., 1., 1., 1.)),
    //                 ZIndex(3)
    //         ));
    //     }  
    // });



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
    return worley_2d((vec.as_vec2() + Vec2::new(offset, offset)) * FREQUENCY, 0.55).x
        + (0.2 * normalised_simplex_noise.powf(BIAS_EXPONENT));
}