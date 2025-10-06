use crate::factory::buildings::buildings::{Building, BuildingData, SpriteResource};
use crate::factory::buildings::Tile;
use crate::factory::{MarkedForRemoval, RemoveBuildingRequest};
use crate::grid::{Grid, GridAtlasSprite, WorldMap};
use crate::ui::interaction::MouseButtonEvent;
use crate::assets::{AtlasId, GameAssets};
use crate::{
    factory::logical::{DataSink, DataSource, LogicalLink},
    grid::{Direction, GridPosition, Orientation},
};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::{Added, With, Without},
    system::{Commands, Query, Res},
};
use bevy::input::gamepad;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component)]
pub struct PhysicalSink(pub Entity, pub Direction);

#[derive(Component)]
pub struct PhysicalSource(pub Entity, pub Direction);

#[derive(Component)]
pub struct PhysicalLink {
    pub throughput: f32,
}

#[derive(Component)]
pub struct Linked;

// ============================================================================
// MESSAGES (Buffered Events)
// ============================================================================

/// Emitted when a PhysicalLink or Building is placed on the grid
#[derive(Event, Message)]
pub struct EntityPlaced {
    pub entity: Entity,
    pub position: GridPosition,
}

/// Emitted when positions need their connections re-evaluated
#[derive(Event, Message)]
pub struct ValidateConnections {
    pub positions: HashSet<GridPosition>,
}

// ============================================================================
// BUILDING IMPLEMENTATION
// ============================================================================

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
            sprite: Some(SpriteResource::Atlas (
                AtlasId::Wires,
                2)), // Default index, will be updated on connection
            grid_width: 1,
            grid_height: 1,
            cost: 25,
            name: "Link".to_string(),
        }
    }
}

// ============================================================================
// PLACEMENT DETECTION SYSTEMS
// ============================================================================

/// Detects newly placed PhysicalLinks and emits EntityPlaced messages
pub fn detect_link_placement(
    query: Query<(Entity, &GridPosition), Added<PhysicalLink>>,
    mut events: MessageWriter<EntityPlaced>,
) {
    for (entity, &position) in query.iter() {
        events.write(EntityPlaced { entity, position });
    }
}

/// Detects newly placed buildings with DataSource/DataSink and emits EntityPlaced messages
pub fn detect_building_placement(
    // Buildings that just got a DataSource or DataSink added
    sources: Query<(Entity, &GridPosition), Added<DataSource>>,
    sinks: Query<(Entity, &GridPosition), Added<DataSink>>,
    mut events: MessageWriter<EntityPlaced>,
) {
    for (entity, &position) in sources.iter() {
        events.write(EntityPlaced { entity, position });
    }
    for (entity, &position) in sinks.iter() {
        events.write(EntityPlaced { entity, position });
    }
}

// ============================================================================
// CONNECTION VALIDATION SYSTEM
// ============================================================================

/// Receives EntityPlaced messages and emits ValidateConnections for affected positions
pub fn validate_placed_entities(
    mut placed_events: MessageReader<EntityPlaced>,
    mut validation_events: MessageWriter<ValidateConnections>,
) {
    let mut positions_to_validate = HashSet::new();

    for event in placed_events.read() {
        // Add the placed entity's position
        positions_to_validate.insert(event.position);

        // Add all neighboring positions
        for (_, neighbor_pos) in event.position.neighbours() {
            positions_to_validate.insert(neighbor_pos);
        }
    }

    if !positions_to_validate.is_empty() {
        validation_events.write(ValidateConnections {
            positions: positions_to_validate,
        });
    }
}

// ============================================================================
// CONNECTION RESOLUTION SYSTEM
// ============================================================================

/// Represents what kind of entity is at a position
#[derive(Debug)]
enum EntityType {
    Link(Entity),
    Source(Entity, Direction),
    Sink(Entity, Direction),
}

/// Main connection resolution system - handles all connection logic
pub fn resolve_connections(
    mut validation_events: MessageReader<ValidateConnections>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
    // Query for PhysicalLinks
    links: Query<(Entity, Option<&PhysicalSink>, Option<&PhysicalSource>), With<PhysicalLink>>,
    // Query for DataSources (on buildings)
    sources: Query<(Entity, &DataSource), (Without<PhysicalLink>, Without<PhysicalSource>)>,
    // Query for DataSinks (on buildings)
    sinks: Query<(Entity, &DataSink), (Without<PhysicalLink>, Without<PhysicalSink>)>,
) {
    for event in validation_events.read() {
        for &position in event.positions.iter() {
            // Get all entities at this position using WorldMap
            let Some(entities_at_pos) = world_map.get(&position) else {
                continue;
            };

            // Check all entities at this position
            for &entity_at_pos in entities_at_pos.iter() {
                // Classify the entity
                let entity_type = classify_entity(entity_at_pos, &links, &sources, &sinks);

                // Check all neighbors and attempt connections
                for (direction, neighbor_pos) in position.neighbours() {
                    let Some(neighbor_entities) = world_map.get(&neighbor_pos) else {
                        continue;
                    };

                    // Try to connect to all entities at the neighbor position
                    for &neighbor_entity in neighbor_entities.iter() {
                        let neighbor_type =
                            classify_entity(neighbor_entity, &links, &sources, &sinks);

                        // Try to connect entity_at_pos -> neighbor
                        attempt_connection(
                            &mut commands,
                            entity_at_pos,
                            neighbor_entity,
                            direction,
                            &entity_type,
                            &neighbor_type,
                            &links,
                        );
                    }
                }
            }
        }
    }
}

/// Classifies an entity into one of the connection types
fn classify_entity(
    entity: Entity,
    links: &Query<(Entity, Option<&PhysicalSink>, Option<&PhysicalSource>), With<PhysicalLink>>,
    sources: &Query<(Entity, &DataSource), (Without<PhysicalLink>, Without<PhysicalSource>)>,
    sinks: &Query<(Entity, &DataSink), (Without<PhysicalLink>, Without<PhysicalSink>)>,
) -> Option<EntityType> {
    // Check if it's a PhysicalLink
    if links.get(entity).is_ok() {
        return Some(EntityType::Link(entity));
    }

    // Check if it's a DataSource (building output)
    if let Ok((_, data_source)) = sources.get(entity) {
        return Some(EntityType::Source(entity, data_source.direction));
    }

    // Check if it's a DataSink (building input)
    if let Ok((_, data_sink)) = sinks.get(entity) {
        return Some(EntityType::Sink(entity, data_sink.direction));
    }

    None
}

/// Attempts to create a connection from source_entity to target_entity
fn attempt_connection(
    commands: &mut Commands,
    source_entity: Entity,
    target_entity: Entity,
    direction_from_source: Direction,
    source_type: &Option<EntityType>,
    target_type: &Option<EntityType>,
    links: &Query<(Entity, Option<&PhysicalSink>, Option<&PhysicalSource>), With<PhysicalLink>>,
) {
    let (Some(source_type), Some(target_type)) = (source_type, target_type) else {
        return;
    };

    match (source_type, target_type) {
        // Building DataSource -> Building DataSink (direct connection)
        (EntityType::Source(source, source_dir), EntityType::Sink(target, target_dir)) => {
            // Check directionality: source must output in direction, sink must input from opposite
            if *source_dir == direction_from_source
                && *target_dir == direction_from_source.opposite()
            {
                insert_physical_connection(commands, *source, *target, direction_from_source);
            }
        }

        // Building DataSource -> PhysicalLink
        (EntityType::Source(source, source_dir), EntityType::Link(target)) => {
            if *source_dir == direction_from_source {
                // Check if link doesn't already have an input
                if let Ok((_, link_sink, _)) = links.get(*target) {
                    if link_sink.is_none() {
                        insert_physical_connection(
                            commands,
                            *source,
                            *target,
                            direction_from_source,
                        );
                    }
                }
            }
        }

        // PhysicalLink -> Building DataSink
        (EntityType::Link(source), EntityType::Sink(target, target_dir)) => {
            if *target_dir == direction_from_source.opposite() {
                // Check if link doesn't already have an output
                if let Ok((_, _, link_source)) = links.get(*source) {
                    if link_source.is_none() {
                        insert_physical_connection(
                            commands,
                            *source,
                            *target,
                            direction_from_source,
                        );
                    }
                }
            }
        }

        // PhysicalLink -> PhysicalLink
        (EntityType::Link(source), EntityType::Link(target)) => {
            if let (Ok((_, source_in, source_out)), Ok((_, target_in, target_out))) =
                (links.get(*source), links.get(*target))
            {
                // Can connect if source has no output and target has no input
                // Also check for cycles
                if source_out.is_none()
                    && target_in.is_none()
                    && !would_create_cycle(*target, *source, links)
                {
                    insert_physical_connection(commands, *source, *target, direction_from_source);
                }
            }
        }

        _ => {
            // No valid connection for this combination
        }
    }
}

/// Checks if adding a connection from -> to would create a cycle
fn would_create_cycle(
    from: Entity,
    to: Entity,
    links: &Query<(Entity, Option<&PhysicalSink>, Option<&PhysicalSource>), With<PhysicalLink>>,
) -> bool {
    let mut current = to;
    let mut seen = HashSet::new();

    while seen.insert(current) {
        match links.get(current) {
            Ok((_, _, Some(source))) => {
                let next = source.0;
                if next == from {
                    return true;
                }
                current = next;
            }
            _ => break,
        }
    }

    false
}

/// Inserts physical connection components on both entities
fn insert_physical_connection(
    commands: &mut Commands,
    source: Entity,
    sink: Entity,
    direction: Direction,
) {
    commands
        .entity(source)
        .insert(PhysicalSource(sink, direction));
    commands
        .entity(sink)
        .insert(PhysicalSink(source, direction));
}

// ============================================================================
// LOGICAL LINK ASSEMBLY SYSTEM
// ============================================================================

/// Assembles logical links for direct building-to-building connections
pub fn assemble_direct_logical_links(
    mut commands: Commands,
    // DataSinks that just got connected (received PhysicalSink)
    newly_connected_sinks: Query<
        (Entity, &PhysicalSink, &DataSink),
        (Without<PhysicalLink>, Added<PhysicalSink>),
    >,
    data_sources: Query<(&DataSource, &PhysicalSource), Without<PhysicalLink>>,
    mut already_linked: Query<&mut LogicalLink>,
) {
    for (sink_entity, physical_sink, data_sink) in newly_connected_sinks.iter() {
        let source_entity = physical_sink.0;

        // Verify the source entity has both DataSource and PhysicalSource
        if let Ok((data_source, physical_source)) = data_sources.get(source_entity) {
            // Verify it's a direct connection (source points to this sink)
            if physical_source.0 == sink_entity {
                // Create a logical link with no intermediate PhysicalLink segments
                let logical_link = LogicalLink {
                    links: Vec::new(), // No PhysicalLink segments for direct connections
                    throughput: data_source.throughput,
                    source: source_entity,
                    sink: sink_entity,
                };

                if let Ok(mut existing) = already_linked.get_mut(sink_entity) {
                    *existing = logical_link;
                } else {
                    commands.entity(sink_entity).insert(logical_link);
                }
            }
        }
    }
}
// pub fn on_physical_link_connected(
//     newly_connected: Query<
//         (Entity, &GridAtlasSprite),
//         (
//             With<PhysicalLink>,
//             Or<(Added<PhysicalSink>, Added<PhysicalSource>)>,
//         )>,

// ) {
//     let 
// }

/// Updates the sprite of PhysicalLinks when they get connected
pub fn update_link_sprite_on_connection(
    mut links: Query<
        (Entity, &mut GridAtlasSprite, &PhysicalSink, &PhysicalSource),
        (
            With<PhysicalLink>,
            Or<(Added<PhysicalSink>, Added<PhysicalSource>)>,
        ),
    >,
    game_assets: Res<GameAssets>,
) {
    for (_entity, mut sprite, sink, source) in links.iter_mut() {
        // Update to wires atlas with appropriate index based on input/output directions
        sprite.atlas_id = AtlasId::Wires;
        sprite.atlas_index = game_assets.wire_index(sink.1, source.1);
    }
}

/// Assembles logical links by walking complete physical chains
pub fn assemble_logical_links(
    mut commands: Commands,
    // PhysicalLinks that just became fully connected (have both input and output)
    newly_connected: Query<
        (Entity, &PhysicalSink, &PhysicalSource),
        (
            With<PhysicalLink>,
            Or<(Added<PhysicalSink>, Added<PhysicalSource>)>,
        ),
    >,
    physical_sinks: Query<&PhysicalSink>,
    physical_sources: Query<&PhysicalSource>,
    physical_links: Query<&PhysicalLink>,
    mut already_linked: Query<&mut LogicalLink>,
) {
    let mut processed = HashSet::new();

    for (entity, sink, source) in newly_connected.iter() {
        if processed.contains(&entity) {
            continue;
        }

        // Walk upstream to find the source endpoint
        let (source_endpoint, mut upstream_chain) = walk_upstream(&physical_sinks, sink.0);

        // Walk downstream to find the sink endpoint
        let (sink_endpoint, mut downstream_chain) = walk_downstream(&physical_sources, source.0);

        let (Some(source_endpoint), Some(sink_endpoint)) = (source_endpoint, sink_endpoint) else {
            continue;
        };

        // Build the full chain
        let mut full_chain = Vec::new();
        full_chain.append(&mut upstream_chain);
        full_chain.push(entity);
        full_chain.append(&mut downstream_chain);

        // Calculate minimum throughput
        let throughput = full_chain
            .iter()
            .filter_map(|&e| physical_links.get(e).ok())
            .map(|link| link.throughput)
            .fold(f32::INFINITY, f32::min);

        // Mark all segments as linked
        for &segment in full_chain.iter() {
            processed.insert(segment);
        }

        // Create or update the logical link on the sink endpoint
        let logical_link = LogicalLink {
            links: full_chain,
            throughput,
            source: source_endpoint,
            sink: sink_endpoint,
        };

        if let Ok(mut existing) = already_linked.get_mut(sink_endpoint) {
            *existing = logical_link;
        } else {
            commands.entity(sink_endpoint).insert(logical_link);
        }
    }
}

/// Walks upstream (following PhysicalSink pointers) to find the source endpoint
fn walk_upstream(sinks: &Query<&PhysicalSink>, start: Entity) -> (Option<Entity>, Vec<Entity>) {
    let mut chain = Vec::new();
    let mut current = start;
    let mut seen = HashSet::new();

    while let Ok(sink) = sinks.get(current) {
        if !seen.insert(current) {
            // Cycle detected
            return (None, Vec::new());
        }
        chain.push(current);
        current = sink.0;
    }

    // current is now the endpoint (doesn't have PhysicalSink)
    (Some(current), chain)
}

/// Walks downstream (following PhysicalSource pointers) to find the sink endpoint
fn walk_downstream(
    sources: &Query<&PhysicalSource>,
    start: Entity,
) -> (Option<Entity>, Vec<Entity>) {
    let mut chain = Vec::new();
    let mut current = start;
    let mut seen = HashSet::new();

    while let Ok(source) = sources.get(current) {
        if !seen.insert(current) {
            // Cycle detected
            return (None, Vec::new());
        }
        chain.push(current);
        current = source.0;
    }

    // current is now the endpoint (doesn't have PhysicalSource)
    (Some(current), chain)
}

// ============================================================================
// CLEANUP ON REMOVAL
// ============================================================================

/// Handles cleanup when a DataSource is removed - removes LogicalLinks that reference it
pub fn on_data_source_removed(
    trigger: On<Remove, DataSource>,
    mut commands: Commands,
    logical_links: Query<(Entity, &LogicalLink)>,
) {
    let removed_entity = trigger.entity;

    // Find and remove any LogicalLinks that have this entity as their source
    for (sink_entity, logical) in logical_links.iter() {
        if logical.source == removed_entity {
            // Remove the LogicalLink from the sink
            if let Ok(mut entity_commands) = commands.get_entity(sink_entity) {
                entity_commands.remove::<LogicalLink>();
            }
        }
    }
}

/// Handles cleanup when a DataSink is removed - removes LogicalLinks that reference it
pub fn on_data_sink_removed(
    trigger: On<Remove, DataSink>,
    mut commands: Commands,
    logical_links: Query<(Entity, &LogicalLink)>,
) {
    // Logical link will clean up itself
}

/// Handles cleanup when a PhysicalLink is removed
pub fn on_physical_link_removed(
    trigger: On<Remove, PhysicalLink>,
    mut commands: Commands,
    physical_sources: Query<(Entity, &PhysicalSource)>,
    physical_sinks: Query<(Entity, &PhysicalSink)>,
    logical_links: Query<(Entity, &LogicalLink)>,
    positions: Query<&GridPosition>,
    mut validation_events: MessageWriter<ValidateConnections>,
) {
    let removed_entity = trigger.entity;
    let mut positions_to_revalidate = HashSet::new();

    // Get the position of the removed entity if still available
    if let Ok(&position) = positions.get(removed_entity) {
        // Add neighbors to revalidation list
        for (_, neighbor_pos) in position.neighbours() {
            positions_to_revalidate.insert(neighbor_pos);
        }
    }

    // Remove physical connections from entities that pointed to the removed entity
    for (owner, source) in physical_sources.iter() {
        if source.0 == removed_entity {
            if let Ok(mut entity_commands) = commands.get_entity(owner) {
                entity_commands.remove::<PhysicalSource>();
            }
            // Add to revalidation
            if let Ok(&pos) = positions.get(owner) {
                positions_to_revalidate.insert(pos);
            }
        }
    }

    for (owner, sink) in physical_sinks.iter() {
        if sink.0 == removed_entity {
            if let Ok(mut entity_commands) = commands.get_entity(owner) {
                entity_commands.remove::<PhysicalSink>();
            }
            // Add to revalidation
            if let Ok(&pos) = positions.get(owner) {
                positions_to_revalidate.insert(pos);
            }
        }
    }

    // Tear down logical links that used this segment
    for (sink_entity, logical) in logical_links.iter() {
        if logical.links.contains(&removed_entity) {
            // Remove the logical link
            if let Ok(mut entity_commands) = commands.get_entity(sink_entity) {
                entity_commands.remove::<LogicalLink>();
            }
        }
    }

    // Emit validation event for affected positions
    if !positions_to_revalidate.is_empty() {
        validation_events.write(ValidateConnections {
            positions: positions_to_revalidate,
        });
    }
}

pub fn remove_physical_link_on_right_click(
    mut commands: Commands,
    mut mouse: ResMut<MouseButtonEvent>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    grid: Res<Grid>,
    world_map: Res<WorldMap>,
    links: Query<&PhysicalLink>,
    tiles: Query<&Tile>,
    mut removal_events: MessageWriter<RemoveBuildingRequest>,
) {
    let Some(mouse) = mouse.handle() else { return };

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

    let grid_pos = grid.world_to_grid(world_pos);

    // Get all entities at this grid position
    let Some(entities) = world_map.get(&grid_pos) else {
        return;
    };

    // Check each entity at this position
    for &entity in entities.iter() {
        // Check if it's a PhysicalLink
        if links.get(entity).is_ok() {
            commands.entity(entity).remove::<PhysicalLink>();
            commands.entity(entity).insert(MarkedForRemoval);
            return; // Stop after removing first PhysicalLink
        }

        // Check if it's a Tile (part of a building)
        if tiles.get(entity).is_ok() {
            removal_events.write(RemoveBuildingRequest { tile: entity });
            return; // Stop after emitting first removal request
        }
    }
}
