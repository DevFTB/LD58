use bevy::prelude::*;

// The resource now holds handles for the atlas texture and its layout.
#[derive(Resource)]
pub struct GameAssets {
    pub icons_texture: Handle<Image>,
    pub icons_layout: Handle<TextureAtlasLayout>,
    pub corporate_icon_index: usize,
    pub government_icon_index: usize,
    pub academia_icon_index: usize,
    pub criminal_icon_index: usize,
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
        corporate_icon_index: 1,
        government_icon_index: 2,
        academia_icon_index: 3,
        criminal_icon_index: 0,
    };
    commands.insert_resource(game_assets);
}

