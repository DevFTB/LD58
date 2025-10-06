use crate::factory::buildings::aggregator::do_aggregation;
use crate::factory::buildings::buildings::Building;
use crate::factory::buildings::combiner::do_combining;
use crate::factory::buildings::delinker::do_delinking;
use crate::factory::buildings::splitter::do_splitting;
use crate::factory::buildings::trunker::do_trunking;
use crate::factory::logical::{
    calculate_throughput, debug_logical_links, pass_data_system, reset_delta, DataSink, DataSource,
};
use crate::factory::physical::{
    connect_direct, connect_links, connect_physical_links_to_data, establish_logical_links,
    on_physical_link_removed,
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
pub mod source_visuals;

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
        use crate::pause::GameState;
        
        app.add_plugins(source_visuals::SourceVisualsPlugin);
        app.add_message::<ConstructBuildingEvent>();
        app.add_observer(on_physical_link_removed);
        // Factory logic systems should only run during normal gameplay
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
                pass_data_system,
                (
                    connect_physical_links_to_data,
                    connect_links,
                    establish_logical_links,
                    connect_direct.run_if(
                        |q1: Query<(), Added<DataSource>>, q2: Query<(), Added<DataSink>>| {
                            !q1.is_empty() || !q2.is_empty()
                        },
                    ),
                    debug_logical_links,
                )
                    .chain(),
            )
                .chain()
                .run_if(in_state(GameState::Running)),
        );
        app.add_systems(
            Update,
            handle_construction_event
                .run_if(in_state(GameState::Running).or(in_state(GameState::ManualPause))),
        );
        app.add_systems(
            PostUpdate,
            (calculate_throughput, reset_delta)
                .chain()
                .run_if(on_timer(Duration::from_secs(1)).and(in_state(GameState::Running))),
        );
    }
}

/// Handles construction events and spawns the appropriate building entity
pub fn handle_construction_event(
    mut construct_events: MessageReader<ConstructBuildingEvent>,
    mut commands: Commands,
    game_assets: Res<crate::assets::GameAssets>,
) {
    for event in construct_events.read() {
        let base_position = GridPosition(event.grid_position);
        // Extract sprite info for all buildings
        event
            .building
            .spawn(&mut commands, base_position, event.orientation);
    }
}

