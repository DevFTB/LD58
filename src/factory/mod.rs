use crate::factory::buildings::aggregator::do_aggregation;
use crate::factory::buildings::combiner::do_combining;
use crate::factory::buildings::delinker::do_delinking;
use crate::factory::buildings::splitter::do_splitting;
use crate::factory::buildings::trunker::do_trunking;
use crate::factory::logical::{debug_logical_links, pass_data_system, visualise_sinks};
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
                (
                    do_delinking,
                    do_aggregation,
                    do_splitting,
                    do_combining,
                    do_trunking,
                ),
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
                connect_direct,
                debug_logical_links,
            )
                .chain(),
        );
    }
}
