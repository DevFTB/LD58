use bevy::prelude::*;

// The resource now holds handles for the atlas texture and its layout.
#[derive(Resource)]
pub struct GameAssets {
    pub icons_texture: Handle<Image>,
    pub icons_layout: Handle<TextureAtlasLayout>,
    pub transparent_icons_texture: Handle<Image>,
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_assets);
    }
}

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("factions/factions.png");
    let transparent_texture_handle = asset_server.load("factions/factions_transparent.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), // The size of each sprite
        2,                    // The number of columns
        2,                    // The number of rows
        None,                 // Optional padding
        None,                 // Optional offset
    );

    let layout_handle = texture_atlas_layouts.add(layout);

    let game_assets = GameAssets {
        icons_texture: texture_handle,
        icons_layout: layout_handle,
        transparent_icons_texture: transparent_texture_handle,
    };
    commands.insert_resource(game_assets);
}

