use crate::GridPosition;
use std::{collections::VecDeque, ops::RangeInclusive};

use bevy::math::I64Vec2;
use bevy::platform::collections::HashMap;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::reflect::Array;
use bevy::render::render_resource::encase::private::Length;
use noisy_bevy::{fbm_simplex_2d_seeded, worley_2d};
use rand::Rng;

use crate::factory::logical::{BasicDataType, DataAttribute, Dataset};

use crate::factions::Faction;
use crate::factory::buildings::sink::SinkBuilding;
use crate::factory::buildings::source::SourceBuilding;
use crate::grid::{Direction, GridSprite};
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalRng;
use rand::prelude::IndexedRandom;

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
    center: I64Vec2,
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
const WORLD_SIZE: i64 = 500;
const WORLD_MIN: i64 = -(WORLD_SIZE / 2);
const WORLD_MAX: i64 = (WORLD_SIZE / 2) - 1;

const STARTING_AREA_SIZE: i64 = 20;
const INITIAL_FACTION_SINKS: [(I64Vec2, Faction); 4] = [
    (I64Vec2::new(0, 7), Faction::Government),
    (I64Vec2::new(7, 0), Faction::Corporate),
    (I64Vec2::new(0, -7), Faction::Criminal),
    (I64Vec2::new(-7, 0), Faction::Academia),
];

// basic sources per 1000 unlocked tiles
const BASIC_SOURCE_DENSITY: i32 = 10;
const SOURCES_PER_FACTION_CLUSTER: RangeInclusive<i32> = 2..=3;

const FACTION_CLUSTER_THRESHOLD: f32 = 0.32;
// check to stop broken clusters from spawning because of start area cutting through them
const MIN_CLUSTER_SIZE: i32 = 6;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let startup_span = info_span!("startup_span", name = "startup_span").entered();
    // apply logic to determine which ones start locked
    let mut unlocked_cells: Vec<I64Vec2> = Vec::new();
    let mut locked_cells: Vec<I64Vec2> = Vec::new();

    // let mut rng = rand::rng();
    let noise_offset: f32 = rng.random_range(-1000.0..1000.0);

    for i in WORLD_MIN..=WORLD_MAX {
        for j in WORLD_MIN..=WORLD_MAX {
            let cell_vec = I64Vec2::new(i, j);
            if in_start_area(cell_vec)
                || get_locked_tile_noise(cell_vec, noise_offset) > FACTION_CLUSTER_THRESHOLD
            {
                unlocked_cells.push(cell_vec);
            } else {
                locked_cells.push(cell_vec);
            }
        }
    }

    // label faction clusters using a bfs
    let mut locked_queue: Vec<I64Vec2> = locked_cells.clone();
    let valid_nodes: HashSet<I64Vec2> = locked_cells.iter().cloned().collect();
    let mut visited: HashSet<I64Vec2> = HashSet::new();

    // grid_pos -> cluster_id
    let mut cluster_map: HashMap<I64Vec2, i64> = HashMap::new();

    // defines free spots to spawn faction sources
    let mut faction_source_locations: HashMap<i64, HashSet<I64Vec2>> = HashMap::new();

    // cluster_id -> grid_pos
    let mut center_map: HashMap<i64, I64Vec2> = HashMap::new();

    let mut cluster_id: i64 = 0;

    while let Some(current) = locked_queue.pop() {
        if visited.insert(current) {
            // new cluster
            let mut queue: VecDeque<I64Vec2> = VecDeque::new();
            let mut cluster_nodes: Vec<I64Vec2> = Vec::new();

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
            // if condition stops tiny clusters formed by start area breaking them up
            if cluster_nodes.length() >= MIN_CLUSTER_SIZE.try_into().unwrap() {
                // found all nodes for current cluster: log cluster id for all nodes and center
                cluster_map.extend(cluster_nodes.iter().cloned().map(|key| (key, cluster_id)));

                faction_source_locations.insert(cluster_id, cluster_nodes.into_iter().collect());

                center_map.insert(cluster_id, center_node.0);

                cluster_id += 1;
            } else {
                // hacky fix to unlock relevant cells
                // the cut still looks funny rthough lol, to fix might be able to apply a falling
                // subtraction on the noise from the center insted of cutting it
                unlocked_cells.extend(cluster_nodes.iter().copied());

                let remove_set: HashSet<I64Vec2> = cluster_nodes.into_iter().collect();
                locked_cells.retain(|e| !remove_set.contains(e));
            }
        }
    }

    println!("cluster map: {:?}", cluster_map);
    println!("center map: {:?}", center_map);

    // Debug visualization disabled for performance - was spawning 100,000+ entities
    // Uncomment only for debugging world generation

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
    let cluster_faction: HashMap<i64, Faction> = HashMap::from(
        center_map
            .iter()
            .map(|(&cluster_id, center_vec)| (cluster_id, map_grid_pos_to_faction(*center_vec)))
            .collect::<HashMap<i64, Faction>>(),
    );

    // map each cluster to a reputation amount
    let cluster_reputation: HashMap<i64, i32> = HashMap::from(
        center_map
            .iter()
            .map(|(&cluster_id, center_vec)| {
                (cluster_id, get_faction_cluster_reputation(*center_vec))
            })
            .collect::<HashMap<i64, i32>>(),
    );

    // debug printing to ensure that gen logic is working
    for (cluster_id, cell_vec) in &center_map {
        if let (Some(faction), Some(reputation)) = (
            cluster_faction.get(cluster_id),
            cluster_reputation.get(cluster_id),
        ) {
            commands.spawn((
                GridPosition(*cell_vec),
                GridSprite(Color::linear_rgba(1., 0.5, 1., 1.)),
                Text2d::new(format!("{:?}: {cluster_id}, rep: {reputation}", faction)),
                ZIndex(4),
            ));
        } else {
            panic!("{cluster_id} has no faction");
        }
    }

    // spawn faction sinks
    // pass faction_source_locations in to remove sink locations from source spawn points
    // super dirty but whatevs
    for (cluster_id, cell_vec) in &center_map {
        if let Some(faction) = cluster_faction.get(cluster_id) {
            // spawn_faction_sink(*cell_vec, *cluster_id, faction.clone(), &cluster_map, &mut commands);
            if let Some(cluster_allowable_spawns) = faction_source_locations.get_mut(cluster_id) {
                spawn_faction_sink(
                    *cell_vec,
                    faction.clone(),
                    Some(&cluster_map),
                    Some(cluster_allowable_spawns),
                    &mut commands,
                );
            }
        } else {
            panic!("{cluster_id} has no faction");
        }
    }

    // spawn intitial faction sinks
    for (position, faction) in INITIAL_FACTION_SINKS {
        spawn_faction_sink(position, faction, Option::None, Option::None, &mut commands);
    }

    let basic_source_amount = (unlocked_cells.length() as i32 / 1000) * BASIC_SOURCE_DENSITY;
    // spawn basic sources
    for cell_vec in
        unlocked_cells.choose_multiple(&mut rng, basic_source_amount.try_into().unwrap())
    {
        // make sure they don't spawn on top of starting sinks: there might be a better way...
        // todo: refactor if necessary
        let mut sink_locs = HashSet::<I64Vec2>::new();

        for (vec, _) in INITIAL_FACTION_SINKS {
            for x in vec.x - 1..=vec.x + 1 {
                for y in vec.y - 1..=vec.y + 1 {
                    sink_locs.insert(I64Vec2::new(x, y));
                }
            }
        }

        if !sink_locs.contains(cell_vec) {
            spawn_source(
                *cell_vec,
                get_basic_source_throughput(*cell_vec),
                get_basic_source_dataset(&mut rng),
                Option::None,
                &mut commands,
            );
        }
    }

    // spawn faction sources
    for cluster_id in center_map.keys() {
        let n_spawns = rng.random_range(SOURCES_PER_FACTION_CLUSTER);
        if let (Some(avaiable_spawns), Some(reputation), Some(faction)) = (
            faction_source_locations.get(cluster_id),
            cluster_reputation.get(cluster_id),
            cluster_faction.get(cluster_id),
        ) {
            spawn_cluster_datasets(
                *cluster_id,
                n_spawns,
                *reputation,
                faction.clone(),
                avaiable_spawns,
                &mut rng,
                &mut commands,
            );
        } else {
            panic!("{cluster_id} missing from a required hashmap")
        }
    }
}

fn spawn_cluster_datasets(
    cluster_id: i64,
    n: i32,
    reputation: i32,
    faction: Faction,
    available_spawns: &HashSet<I64Vec2>,
    rng: &mut WyRand,
    commands: &mut Commands,
) {
    let dataset = get_faction_source_dataset(faction.clone(), reputation, rng);
    let throughput = get_faction_source_throughput(reputation);

    for cell_vec in available_spawns
        .into_iter()
        .copied()
        .collect::<Vec<I64Vec2>>()
        .choose_multiple(rng, n.try_into().unwrap())
    {
        spawn_source(
            *cell_vec,
            throughput,
            dataset.clone(),
            Some(faction.clone()),
            commands,
        );
    }
}

fn get_basic_source_dataset(rng: &mut WyRand) -> Dataset {
    let basic_datasets: [Dataset; 4] = [
        Dataset {
            contents: HashMap::from([(BasicDataType::Biometric, HashSet::<DataAttribute>::new())]),
        },
        Dataset {
            contents: HashMap::from([(
                BasicDataType::Behavioural,
                HashSet::<DataAttribute>::new(),
            )]),
        },
        Dataset {
            contents: HashMap::from([(BasicDataType::Economic, HashSet::<DataAttribute>::new())]),
        },
        Dataset {
            contents: HashMap::from([(BasicDataType::Telemetry, HashSet::<DataAttribute>::new())]),
        },
    ];

    if let Some(chosen_dataset) = basic_datasets.choose(rng) {
        return chosen_dataset.clone();
    } else {
        panic!("no basic source dataset or choose broken")
    }
}

fn get_basic_source_throughput(vec: I64Vec2) -> f32 {
    // TODO: introduce some randomness if desired
    let length_f64_squared = vec.length_squared() as f64;
    let length_f64 = length_f64_squared.sqrt();
    if length_f64 <= 40. {
        50.
    } else if length_f64 <= 60. {
        100.
    } else if length_f64 <= 100. {
        150.
    } else if length_f64 <= 150. {
        200.
    } else {
        250.
    }
}

fn get_faction_source_throughput(reputation: i32) -> f32 {
    // TODO: introduce some randomness if desired
    match reputation {
        6 => 400.,
        5 => 300.,
        4 => 150.,
        _ => 50.,
    }
}

fn get_faction_source_dataset(faction: Faction, reputation: i32, rng: &mut WyRand) -> Dataset {
    return Dataset {
        contents: HashMap::from([(BasicDataType::Biometric, HashSet::<DataAttribute>::new())]),
    };
}

fn get_faction_cluster_reputation(vec: I64Vec2) -> i32 {
    let length_f64_squared = vec.length_squared() as f64;
    let length_f64 = length_f64_squared.sqrt();
    if length_f64 <= 80. {
        4
    } else if length_f64 <= 130. {
        5
    } else {
        6
    }
}

fn spawn_source(
    vec: I64Vec2,
    throughput: f32,
    dataset: Dataset,
    faction: Option<Faction>,
    commands: &mut Commands,
) {
    let source = SourceBuilding::get_bundle(
        vec.into(),
        Direction::ALL.to_vec(),
        dataset.clone(),
        throughput,
        false,
    );

    let mut entity = commands.spawn((
        source,
        ZIndex(3),
        Text2d::new(format!("{:?}: {throughput}", dataset.clone())),
    ));

    if let Some(actual_faction) = faction {
        entity.insert(FactionComponent(actual_faction));
    }
}

fn in_start_area(vec: I64Vec2) -> bool {
    return vec.length_squared() < STARTING_AREA_SIZE.pow(2);
}

// this option implementation is sus and hack refactor later lol
// this whole funciton is so sus shahahahahahah
fn spawn_faction_sink(
    position: I64Vec2,
    faction: Faction,
    cluster_map: Option<&HashMap<I64Vec2, i64>>,
    cluster_hash_set: Option<&mut HashSet<I64Vec2>>,
    commands: &mut Commands,
) {
    let mut sink_parent = commands.spawn((
        SinkParent,
        // ClusterID(cluster_id),
        FactionComponent(faction),
    ));

    let mut sink_vecs: Vec<I64Vec2> = Vec::new();
    for x in position.x - 1..=position.x + 1 {
        for y in position.y - 1..=position.y + 1 {
            let cur_vec = I64Vec2::new(x, y);
            if let Some(cluster_map_val) = cluster_map {
                if cluster_map_val.contains_key(&cur_vec) {
                    sink_vecs.push(cur_vec);
                }
            } else {
                sink_vecs.push(cur_vec);
            }
        }
    }

    for grid_vec in &sink_vecs {
        commands.spawn((
            GridPosition(*grid_vec),
            GridSprite(Color::linear_rgba(1., 1., 1., 1.)),
            ZIndex(3),
        ));
    }

    // remove sink location from allowable source spawn locations
    if let Some(cluster_hash_set_val) = cluster_hash_set {
        let remove_set: HashSet<I64Vec2> = sink_vecs.into_iter().collect();
        cluster_hash_set_val.retain(|e| !remove_set.contains(e));
    }

    let sink = SinkBuilding::get_sized_bundle(position.into(), 2, None);
    commands.spawn(sink);
}

fn map_grid_pos_to_faction(vec: I64Vec2) -> Faction {
    let y = vec.y;
    let x = vec.x;
    return match (y >= x, y >= -x) {
        // top
        (true, true) => Faction::Government,
        // right
        (false, true) => Faction::Corporate,
        // bottom
        (false, false) => Faction::Criminal,
        // left
        (true, false) => Faction::Academia,
    };
}

fn get_locked_tile_noise(vec: I64Vec2, offset: f32) -> f32 {
    const SIMPLEX_FREQUENCY: f32 = 0.035;
    const BIAS_EXPONENT: f32 = 2.0;
    let normalised_simplex_noise =
        (fbm_simplex_2d_seeded(vec.as_vec2() * SIMPLEX_FREQUENCY, 2, 2., 0.1, 48.) + 1.0) / 2.0;

    const FREQUENCY: f32 = 0.035;
    return worley_2d(
        (vec.as_vec2() + Vec2::new(offset, offset)) * FREQUENCY,
        0.55,
    )
    .x + (0.2 * normalised_simplex_noise.powf(BIAS_EXPONENT));
}
