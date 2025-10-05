use crate::factory::buildings::aggregator::do_aggregation;
use crate::factory::buildings::combiner::do_combining;
use crate::factory::buildings::delinker::do_delinking;
use crate::factory::buildings::splitter::do_splitting;
use crate::factory::buildings::trunker::do_trunking;
use crate::factory::logical::{
    debug_logical_links, pass_data_system, visualise_sinks, DataSink, DataSource,
};
use crate::factory::physical::{
    connect_direct, connect_links, connect_physical_links_to_data, establish_logical_links,
    on_physical_link_removed,
};
use crate::factory::buildings::buildings::{BuildingType, BuildingSpecificData};
use crate::factory::buildings::splitter::Splitter;
use crate::grid::{Direction, GridPosition};
use bevy::app::Update;
use bevy::prelude::{Added, Query};
use bevy::{
    app::{Plugin, PostUpdate},
    ecs::schedule::IntoScheduleConfigs,
    prelude::*,
};

pub mod buildings;
pub mod logical;
pub mod physical;
pub struct FactoryPlugin;

/// Event for constructing a building
#[derive(Event, Message)]
pub struct ConstructBuildingEvent {
    pub building_type: BuildingType,
    pub grid_position: bevy::math::I64Vec2,
    pub direction: Direction,
}

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<ConstructBuildingEvent>();
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
                visualise_sinks,
            )
                .chain(),
        );
        app.add_systems(
            PostUpdate,
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
        );
    }
}

/// Handles construction events and spawns the appropriate building entity
pub fn handle_construction_event(
    mut construct_events: MessageReader<ConstructBuildingEvent>,
    mut commands: Commands,
) {
    for event in construct_events.read() {
        let data = event.building_type.data();
        let base_position = GridPosition(event.grid_position);

        match event.building_type {
            BuildingType::Splitter2x1 | BuildingType::Splitter3x1 | BuildingType::Splitter4x1 => {
                // Splitters use the direction as the source direction
                if let BuildingSpecificData::Splitter { outputs: _outputs, loss_rate: _loss_rate } = data.specific {
                    let throughput = 50.0; // Default throughput
                    commands.spawn(Splitter::get_bundle(
                        base_position,
                        throughput,
                        event.direction,
                    ));
                }
            }
            BuildingType::Decoupler => {
                // TODO: Implement Decoupler when the module is ready
                info!("Decoupler construction not yet implemented");
            }
            BuildingType::Collector => {
                // TODO: Implement Collector
                info!("Collector construction not yet implemented");
            }
            BuildingType::Aggregator => {
                // TODO: Implement Aggregator
                info!("Aggregator construction not yet implemented");
            }
            BuildingType::Link => {
                // TODO: Implement Link
                info!("Link construction not yet implemented");
            }
            BuildingType::Combiner => {
                // TODO: Implement Combiner
                info!("Combiner construction not yet implemented");
            }
        }
    }
}
