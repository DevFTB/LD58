extern crate core;

use bevy_prng::WyRand;
use bevy_rand::prelude::*;

use crate::player::PlayerPlugin;
use crate::ui::interaction::CustomInteractionPlugin;
use crate::ui::tooltip::inherit_translation;
use crate::world_gen::WorldGenPlugin;
use crate::{
    assets::AssetPlugin,
    camera::GameCameraPlugin,
    contracts::ContractsPlugin,
    events::EventsPlugin,
    factions::FactionsPlugin,
    factory::FactoryPlugin,
    grid::{GridPlugin, GridPosition},
    ui::UIPlugin,
    pause::PausePlugin,
};
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

mod assets;
mod camera;
mod contracts;
mod events;
mod factions;
mod factory;
mod grid;
mod player;
mod test;
mod ui;
mod world_gen;
mod pause;

fn main() {    
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.25, 0.25, 0.25)))
        .add_plugins(AssetPlugin)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins(PausePlugin)
        .add_plugins(EventsPlugin)
        .add_plugins(ContractsPlugin)
        .add_plugins(GameCameraPlugin)
        .add_plugins(WorldGenPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(FactoryPlugin)
        .add_plugins(FactionsPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(CustomInteractionPlugin)
        .add_systems(Startup, startup)
        .add_systems(PostUpdate, inherit_translation)
        .run();
}

fn startup(_commands: Commands) {
    //test::spawn_splitter_test(&mut commands);
    //test::spawn_delinker_test(&mut commands);
    //test::spawn_combiner_test(&mut commands);
    //test::spawn_trunking_test(&mut commands);
    //test::spawn_sized_sink_test(&mut commands);
}

#[derive(Component, Deref)]
#[component(on_remove = cleanup_linked_spawn)]
pub struct LinkedSpawn(Vec<Entity>);

fn cleanup_linked_spawn(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    // Get the LinkedSpawn component data before it's removed
    if let Some(linked_spawn) = world.get::<LinkedSpawn>(entity) {
        // Clone the entity list before we drop the borrow
        let entities_to_despawn = linked_spawn.0.clone();

        // Despawn all linked entities using commands
        for &linked_entity in &entities_to_despawn {
            world.commands().entity(linked_entity).despawn();
        }
    }
}
