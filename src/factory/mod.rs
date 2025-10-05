use crate::factory::buildings::aggregator::do_aggregation;
use crate::factory::logical::pass_data_system;
use crate::factory::physical::{
    connect_direct, connect_links, connect_physical_links_to_data, establish_logical_links,
    on_physical_link_removed,
};
use bevy::app::Update;
use bevy::{
    app::{Plugin, PostUpdate},
    ecs::schedule::IntoScheduleConfigs,
};

pub mod buildings;
pub mod logical;
pub mod physical;
pub struct FactoryPlugin;

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_observer(on_physical_link_removed);
        app.add_systems(
            Update,
            (
                pass_data_system,
                do_aggregation,
                // debug_sinks
            )
                .chain(),
        );
        app.add_systems(
            PostUpdate,
            (
                connect_physical_links_to_data,
                connect_links,
                establish_logical_links,
                connect_direct,
                // debug_logical_links,
            )
                .chain(),
        );
    }
}
