use crate::factory::buildings::buildings::{Building, BuildingData, SpriteResource};
use crate::factory::buildings::Tile;
use crate::grid::GridAtlasSprite;
use crate::assets::GameAssets;
use crate::{
    factory::logical::{DataSink, DataSource, LogicalLink},
    grid::{Direction, GridPosition, Orientation},
};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{Added, With, Without},
    system::{Commands, Query},
};
use bevy::input::gamepad;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use itertools::Itertools;

#[derive(Component)]
pub struct PhysicalSink(Entity, Direction);

#[derive(Component)]
pub struct PhysicalSource(Entity, Direction);

#[derive(Component)]
pub struct PhysicalLink {
    pub throughput: f32,
}

impl Building for PhysicalLink {
    fn spawn_naked(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        _: Orientation,
    ) -> Entity {
        commands
            .spawn((PhysicalLink { throughput: 234.0 }, position))
            .with_related::<Tile>(())
            .id()
    }

    fn spawn(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        let id = self.spawn_naked(commands, position, orientation);
        let data = self.data();

        match data.sprite {
            Some(SpriteResource::Atlas(atlas_id, index)) => {
                commands.entity(id).insert(GridAtlasSprite {
                    atlas_id,
                    atlas_index: index,
                    grid_width: data.grid_width,
                    grid_height: data.grid_height,
                    orientation,
                });
            }
            Some(SpriteResource::Machine(machine_type, variant)) => {
                // Convert Machine to Atlas using deferred command like in buildings.rs
                let grid_width = data.grid_width;
                let grid_height = data.grid_height;
                commands.queue(move |world: &mut World| {
                    if let Some(game_assets) = world.get_resource::<crate::assets::GameAssets>() {
                        if let Some((atlas_id, index)) = game_assets.machine_sprite(machine_type, variant) {
                            if let Ok(mut entity) = world.get_entity_mut(id) {
                                entity.insert(GridAtlasSprite {
                                    atlas_id,
                                    atlas_index: index,
                                    grid_width,
                                    grid_height,
                                    orientation,
                                });
                            }
                        }
                    }
                });
            }
            Some(SpriteResource::Sprite(image)) => {
                commands.entity(id).insert(Sprite { image, ..default() });
            }
            None => {}
        };

        id
    }
    fn data(&self) -> BuildingData {
        BuildingData {
            sprite: Some(SpriteResource::Atlas(crate::assets::AtlasId::Buildings1x1, 2)),
            grid_width: 1,
            grid_height: 1,
            cost: 25,
            name: "Link".to_string(),
        }
    }
}

#[derive(Component)]
pub struct Linked;
pub fn connect_physical_links_to_data(
    query: Query<(Entity, &GridPosition), Added<PhysicalLink>>,
    mut commands: Commands,
    outputs: Query<(Entity, &GridPosition, &DataSource), Without<PhysicalSource>>,
    inputs: Query<(Entity, &GridPosition, &DataSink), Without<PhysicalSink>>,
) {
    for (entity, new_grid_position) in query.iter() {
        let neighbours = new_grid_position.neighbours();
        // Determine directionality in input
        let candidate = outputs
            .iter()
            .filter_map(|(input_entity, grid_pos, output)| {
                neighbours
                    .iter()
                    .find(|(dir, pos)| grid_pos == pos && output.direction == dir.opposite())
                    .map(|(dir, _)| (input_entity, dir))
            })
            .next();

        if let Some((neighbour_entity, dir)) = candidate {
            insert_physical_connection(&mut commands, neighbour_entity, entity, *dir);
        }
    }

    for (entity, new_grid_position) in query.iter() {
        let neighbours = new_grid_position.neighbours();
        // Determine directionality in input
        let candidate = inputs
            .iter()
            .filter_map(|(input_entity, grid_pos, input)| {
                neighbours
                    .iter()
                    .find(|(dir, pos)| grid_pos == pos && input.direction == dir.opposite())
                    .map(|(dir, _)| (input_entity, dir))
            })
            .next();

        if let Some((neighbour_entity, dir)) = candidate {
            insert_physical_connection(&mut commands, entity, neighbour_entity, *dir);
        }
    }
}

pub fn connect_direct(
    mut commands: Commands,
    all_sources: Query<
        (Entity, &GridPosition, &DataSource),
        (Without<PhysicalSource>, Without<DataSink>),
    >,
    all_sinks: Query<
        (Entity, &GridPosition, &DataSink),
        (Without<PhysicalSink>, Without<DataSource>, Added<DataSink>),
    >,
    existing_links: Query<&LogicalLink>,
) {
    let mut linked_sinks = HashSet::new();

    let source_map = all_sources.iter().into_group_map_by(|x| x.1);

    for (sink_entity, sink_pos, sink) in all_sinks.iter() {
        if existing_links.get(sink_entity).is_ok() {
            continue;
        }
        if linked_sinks.contains(&sink_entity) {
            continue;
        }

        for (dir_from_sink, pos) in sink_pos.neighbours().into_iter() {
            if let Some(vec) = source_map.get(&pos) {
                let dir_from_source = dir_from_sink.opposite();
                let Some((source_entity, _, source)) = vec.iter().find(|(_, _, source)| {
                    source.direction == dir_from_source && sink.direction == dir_from_sink
                }) else {
                    continue;
                };

                if source.direction == dir_from_source && sink.direction == dir_from_sink {
                    let link = LogicalLink {
                        links: Vec::new(),
                        throughput: source.throughput,
                        source: *source_entity,
                        sink: sink_entity,
                    };

                    insert_physical_connection(
                        &mut commands,
                        *source_entity,
                        sink_entity,
                        dir_from_source,
                    );
                    commands.entity(sink_entity).insert(link);
                    linked_sinks.insert(sink_entity);
                    break;
                }
            }
        }
    }
}

pub fn connect_links(
    mut commands: Commands,
    new_links: Query<Entity, Added<PhysicalLink>>,
    // Merge the info we need into one query to avoid repeatedly looking up in multiple queries.
    links: Query<
        (
            Entity,
            &GridPosition,
            Option<&PhysicalSink>,
            Option<&PhysicalSource>,
        ),
        With<PhysicalLink>,
    >,
) {
    // Deterministic pass order is often helpful for debugging.
    let mut to_process: Vec<Entity> = new_links.iter().collect();
    to_process.sort_unstable();

    // Index all positions -> entities for O(1) neighbor lookup.
    let mut pos_to_entity = HashMap::new();
    for (e, pos, _, _) in links.iter() {
        pos_to_entity.insert(*pos, e);
    }

    for &me in &to_process {
        let (me, me_pos, me_in, me_out) = match links.get(me) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Find present neighbors by checking the indexed positions.
        let neighbors: Vec<(Entity, Direction)> = me_pos
            .neighbours() // Iterator over (Direction, GridPosition)
            .into_iter()
            .filter_map(|(dir, p)| pos_to_entity.get(&p).copied().map(|n| (n, dir)))
            .collect();

        for (nbr, dir_to_nbr) in neighbors {
            // Grab neighbor IO state
            let (_, _, nbr_in, nbr_out) = match links.get(nbr) {
                Ok(t) => t,
                Err(_) => continue,
            };

            // 1) Try me -> neighbor
            if me_out.is_none() && nbr_in.is_none() && !would_create_cycle(nbr, me, &links) {
                insert_physical_connection(&mut commands, me, nbr, dir_to_nbr);
                continue;
            }

            // 2) Try neighbor -> me
            if nbr_out.is_none() && me_in.is_none() && !would_create_cycle(me, nbr, &links) {
                // If you have Direction::opposite(), prefer using it. Otherwise compute reverse dir as needed.
                let back_dir = dir_to_nbr.opposite();
                insert_physical_connection(&mut commands, nbr, me, back_dir);
            }
        }
    }
}

// Returns true if adding an edge `from -> to` would create a cycle.
// We check: is there already a path to -> ... -> from following outputs?
fn would_create_cycle(
    from: Entity,
    to: Entity,
    links: &Query<
        (
            Entity,
            &GridPosition,
            Option<&PhysicalSink>,
            Option<&PhysicalSource>,
        ),
        With<PhysicalLink>,
    >,
) -> bool {
    // Walk from `to` following outputs; if we can reach `from`, adding `from -> to` closes a cycle.
    let mut current = to;
    let mut seen = HashSet::new();
    while seen.insert(current) {
        match links.get(current) {
            Ok((_, _, _, Some(out))) => {
                let next = out.0; // PhysicalOutput(target_entity, Direction)
                if next == from {
                    return true;
                }
                current = next;
            }
            _ => break, // no output -> path ends
        }
    }
    false
}
fn insert_physical_connection(
    commands: &mut Commands,
    source: Entity,
    sink: Entity,
    dir: Direction,
) {
    commands.entity(source).insert(PhysicalSource(sink, dir));
    commands.entity(sink).insert(PhysicalSink(source, dir));
}

pub fn on_physical_link_removed(
    trigger: On<Remove, PhysicalLink>,
    mut commands: Commands,
    q_sources: Query<(Entity, &PhysicalSource)>,
    q_sinks: Query<(Entity, &PhysicalSink)>,
    q_logicals: Query<(Entity, &LogicalLink)>, // lives on sink endpoints in your setup
) {
    let broken = trigger.entity;

    // 1) Cut physical pointers from neighbors that referenced the broken segment
    for (owner, src) in &q_sources {
        if src.0 == broken {
            if let Ok(mut e) = commands.get_entity(owner) {
                e.remove::<PhysicalSource>();
            }
        }
    }
    for (owner, sink) in &q_sinks {
        if sink.0 == broken {
            if let Ok(mut e) = commands.get_entity(owner) {
                e.remove::<PhysicalSink>();
            }
        }
    }

    // 2) Tear down any LogicalLink that traversed the broken segment
    for (sink_entity, logical) in &q_logicals {
        if logical.links.iter().any(|&seg| seg == broken) {
            // Unmark all segments that still exist
            for &seg in &logical.links {
                if let Ok(mut e) = commands.get_entity(seg) {
                    e.remove::<Linked>();
                }
            }
            // Remove the logical link from its sink if it still exists
            if let Ok(mut e) = commands.get_entity(sink_entity) {
                e.remove::<LogicalLink>();
            }
        }
    }

    // Do not try to modify the broken entity itself; it may be despawned.
}

pub fn establish_logical_links(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<PhysicalSink>,
            With<PhysicalSource>,
            Added<PhysicalLink>,
        ),
    >,
    inputs: Query<&PhysicalSink>,
    outputs: Query<&PhysicalSource>,
    links: Query<&PhysicalLink>,
) {
    let mut dirty = HashSet::<Entity>::new();

    for entity in query.iter() {
        if dirty.contains(&entity) {
            continue;
        }
        if let (Ok(PhysicalSink(next_input, _)), Ok(PhysicalSource(next_output, _))) =
            (inputs.get(entity), outputs.get(entity))
        {
            let (source_entity, mut upstream_links) = walk_chain(&inputs, Some(*next_input));
            // Walk downstream (toward the sink) by following PhysicalOutput pointers.
            let (sink_entity, mut downstream_links) = walk_chain(&outputs, Some(*next_output));

            let (Some(sink_entity), Some(source_entity)) = (sink_entity, source_entity) else {
                continue;
            };

            let mut full_links = Vec::<Entity>::new();
            full_links.append(&mut upstream_links);
            full_links.push(entity);
            full_links.append(&mut downstream_links);

            let throughput = full_links
                .iter()
                .map(|e| links.get(*e).unwrap().throughput)
                .reduce(f32::min)
                .unwrap_or(0.);

            for link in full_links.iter() {
                commands.entity(*link).insert(Linked);
                dirty.insert(link.clone());
            }

            let link = LogicalLink {
                links: full_links,
                throughput,
                source: source_entity,
                sink: sink_entity,
            };
            commands.entity(sink_entity).insert(link);
        }
    }
}
trait NextHop {
    fn next(&self) -> Entity;
}
impl NextHop for PhysicalSink {
    fn next(&self) -> Entity {
        self.0
    }
}
impl NextHop for PhysicalSource {
    fn next(&self) -> Entity {
        self.0
    }
}

fn walk_chain<C: NextHop + bevy::prelude::Component>(
    q: &Query<&C>,
    start: Option<Entity>,
) -> (Option<Entity>, Vec<Entity>) {
    let Some(mut curr) = start else {
        return (None, Vec::new());
    };

    let mut nodes = Vec::new(); // includes start and every hop we follow
    let mut seen = HashSet::new(); // cycle guard

    nodes.push(curr);
    seen.insert(curr);

    // Keep following while the current node has component C
    while let Ok(comp) = q.get(curr) {
        let next = comp.next();
        if !seen.insert(next) {
            // Cycle detected; bail out
            return (None, Vec::new());
        }
        nodes.push(next);
        curr = next;
    }

    // `curr` is the terminal endpoint that doesn't have C
    // nodes = [start, ..., endpoint]; remove endpoint from chain
    let endpoint = nodes.pop();
    (endpoint, nodes) // nodes = chain of PhysicalLink segments on this side
}
