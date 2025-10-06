use crate::factory::buildings::aggregator::do_aggregation;
use crate::factory::buildings::buildings::Building;
use crate::factory::buildings::combiner::do_combining;
use crate::factory::buildings::delinker::do_delinking;
use crate::factory::buildings::splitter::do_splitting;
use crate::factory::buildings::trunker::do_trunking;
use crate::factory::logical::{
    DataSink, DataSource, calculate_throughput, debug_logical_links, pass_data_system, reset_delta,
};
use crate::factory::physical::{
    EntityPlaced, ValidateConnections, assemble_direct_logical_links, assemble_logical_links,
    detect_building_placement, detect_link_placement, on_physical_link_removed,
    resolve_connections, validate_placed_entities,
};
use crate::grid::{GridPosition, Orientation};
use bevy::time::common_conditions::on_timer;
use bevy::{
    app::{Plugin, Update},
    ecs::schedule::IntoScheduleConfigs,
    math::I64Vec2,
    prelude::*,
};
use std::sync::Arc;
use std::time::Duration;

pub mod buildings;
pub mod logical;
pub mod physical;
pub struct FactoryPlugin;

/// Event for constructing a building
#[derive(Event, Message)]
pub struct ConstructBuildingEvent {
    pub building: Arc<dyn Building>,
    pub grid_position: I64Vec2,
    pub orientation: Orientation,
}

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<ConstructBuildingEvent>();

        // Register new messages for the message-based physical connection system
        app.add_message::<EntityPlaced>();
        app.add_message::<ValidateConnections>();

        app.add_observer(on_physical_link_removed);
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
            )
                .chain(),
        );
        app.add_systems(
            PostUpdate,
            (calculate_throughput, reset_delta)
                .chain()
                .run_if(on_timer(Duration::from_secs(1))),
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
