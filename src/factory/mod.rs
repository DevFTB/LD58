use bevy::{
    app::{Plugin, PostUpdate},
    ecs::schedule::IntoScheduleConfigs,
};
use bevy::app::Update;
use crate::factory::logical::pass_data;
use crate::factory::physical::{connect_links, connect_physical_inputs, connect_physical_outputs, establish_logical_links};

pub mod logical;
pub mod physical;
pub struct FactoryPlugin;

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, pass_data);
        app.add_systems(
            PostUpdate,
            (connect_physical_inputs, connect_physical_outputs, connect_links, establish_logical_links).chain(),
        );
    }
}
