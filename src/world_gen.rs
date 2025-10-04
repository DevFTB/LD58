use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, BROWN, GRAY}, ecs::error::info, prelude::*
};
use noisy_bevy::simplex_noise_2d;
use super::Faction;

pub struct WorldGenPlugin;

#[derive(Component)]
pub struct Locked;

#[derive(Component)]
#[require(Faction)]
pub struct FactionSquare;


impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, startup);
    }
}
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {

}

fn get_locked_tile_noise(vec: Vec2) -> f32{
    return simplex_noise_2d(vec);
}