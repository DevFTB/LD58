use crate::factory::buildings::aggregator::do_aggregation;
use crate::factory::buildings::buildings::Building;
use crate::factory::buildings::combiner::do_combining;
use crate::factory::buildings::delinker::do_delinking;
use crate::factory::buildings::splitter::do_splitting;
use crate::factory::buildings::trunker::do_trunking;
use crate::factory::buildings::sink::{update_sink_throughput, update_sink_debug_text};
use crate::factory::buildings::Undeletable;
use crate::factory::logical::{
    calculate_throughput, debug_logical_links, pass_data_system, reset_delta,
};
use crate::factory::physical::{
    assemble_direct_logical_links, assemble_logical_links, detect_building_placement, detect_link_placement,
    on_data_sink_removed, on_data_source_removed, on_physical_link_removed, resolve_connections,
    validate_placed_entities, EntityPlaced, ValidateConnections,
};
use crate::grid::{GridPosition, Orientation};
use bevy::ecs::relationship::Relationship;
use bevy::time::common_conditions::on_timer;
use bevy::{
    app::{Plugin, PostUpdate, Update},
    ecs::schedule::IntoScheduleConfigs,
    math::I64Vec2,
    prelude::*,
};
use physical::remove_physical_link_on_right_click;
use std::sync::Arc;
use std::time::Duration;

pub mod buildings;
pub mod logical;
pub mod physical;
pub struct FactoryPlugin;

/// Component marking an entity for removal in PostUpdate
#[derive(Component)]
pub struct MarkedForRemoval;

/// Event for constructing a building
#[derive(Event, Message)]
pub struct ConstructBuildingEvent {
    pub building: Arc<dyn Building>,
    pub grid_position: I64Vec2,
    pub orientation: Orientation,
}

/// Event for removing a building when a tile is clicked
#[derive(Event, Message)]
pub struct RemoveBuildingRequest {
    pub tile: Entity,
}

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<ConstructBuildingEvent>();
        app.add_message::<RemoveBuildingRequest>();

        // Register new messages for the message-based physical connection system
        app.add_message::<EntityPlaced>();
        app.add_message::<ValidateConnections>();

        app.add_observer(on_physical_link_removed);
        app.add_observer(on_data_source_removed);
        app.add_observer(on_data_sink_removed);
        app.add_systems(
            Update,
            (
                (
                    do_delinking,
                    do_aggregation,
                    do_splitting,
                    do_combining,
                    do_trunking,
                ),
                handle_construction_event,
                pass_data_system,
                (
                    // New event-based connection system
                    detect_link_placement,
                    detect_building_placement,
                    validate_placed_entities,
                    resolve_connections,
                    assemble_direct_logical_links,
                    assemble_logical_links,
                    debug_logical_links,
                )
                    .chain(),
                update_sink_throughput,
                update_sink_debug_text,
            )
                .chain(),
        );
        app.add_systems(
            PostUpdate,
            (
                (calculate_throughput, reset_delta)
                    .chain()
                    .run_if(on_timer(Duration::from_secs(1))),
                process_entity_removal,
            ),
        );
        app.add_systems(
            Update,
            (remove_physical_link_on_right_click, handle_building_removal),
        );
    }
}

/// Handles construction events and spawns the appropriate building entity
pub fn handle_construction_event(
    mut construct_events: MessageReader<ConstructBuildingEvent>,
    mut commands: Commands,
) {
    for event in construct_events.read() {
        let base_position = GridPosition(event.grid_position);
        // Extract sprite info for all buildings
        event
            .building
            .spawn(&mut commands, base_position, event.orientation);
    }
}

/// Handles building removal requests by marking entities for removal
pub fn handle_building_removal(
    mut events: MessageReader<RemoveBuildingRequest>,
    mut commands: Commands,
    tiles_query: Query<&buildings::Tile, Without<Undeletable>>,
    parent_tiles_query: Query<&buildings::Tiles, Without<Undeletable>>,
) {
    for event in events.read() {
        // Get the parent building from the clicked tile
        if let Ok(tile) = tiles_query.get(event.tile) {
            let parent = tile.get();

            // Get all tiles from the parent
            if let Ok(all_tiles) = parent_tiles_query.get(parent) {
                // Mark the parent building for removal
                commands.entity(parent).insert(MarkedForRemoval);
            }
        }
    }
}

pub fn process_entity_removal(
    mut commands: Commands,
    marked_entities: Query<Entity, With<MarkedForRemoval>>,
) {
    for entity in marked_entities.iter() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
}
