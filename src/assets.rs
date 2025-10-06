use bevy::math::URect;
use bevy::prelude::*;

// Constants for the buildings spritesheet grid
const TILE_SIZE: u32 = 32; // Each grid cell is 64x64 pixels
const SHEET_COLUMNS: u32 = 12; // Total columns in the spritesheet (adjust to your sheet)
const SHEET_ROWS: u32 = 8; // Total rows in the spritesheet (adjust to your sheet)

// The resource now holds handles for the atlas texture and its layout.
#[derive(Resource)]
pub struct GameAssets {
    pub icons_texture: Handle<Image>,
    pub icons_layout: Handle<TextureAtlasLayout>,
    pub transparent_icons_texture: Handle<Image>,
    pub buildings_texture: Handle<Image>,
    pub buildings_layout: Handle<TextureAtlasLayout>,
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_assets);
    }
}

/// Helper function to create a rect from grid coordinates
/// - `col`: Starting column (0-indexed)
/// - `row`: Starting row (0-indexed)
/// - `width`: Width in grid cells (e.g., 2 for a 2x1 building)
/// - `height`: Height in grid cells (e.g., 1 for a 2x1 building)
fn grid_rect(col: u32, row: u32, width: u32, height: u32) -> URect {
    let min_x = col * TILE_SIZE;
    let min_y = row * TILE_SIZE;
    let max_x = min_x + (width * TILE_SIZE);
    let max_y = min_y + (height * TILE_SIZE);

    URect::new(min_x, min_y, max_x, max_y)
}

/// Creates the texture atlas layout for buildings with variable sizes
/// Add your sprites here using grid_rect(column, row, width, height)
fn create_buildings_layout() -> TextureAtlasLayout {
    let sheet_size = UVec2::new(SHEET_COLUMNS * TILE_SIZE, SHEET_ROWS * TILE_SIZE);
    let mut layout = TextureAtlasLayout::new_empty(sheet_size);

    // Example layout - UPDATE THESE to match your actual spritesheet!
    // Index 0: Collector (1x1) at grid position (0, 0)
    layout.add_texture(grid_rect(0, 0, 1, 1));

    // Index 1: Aggregator (1x1) at grid position (1, 0)
    layout.add_texture(grid_rect(4, 0, 1, 1));

    // Index 2: Link (1x1) at grid position (2, 0)
    layout.add_texture(grid_rect(12, 0, 1, 1));

    // Index 3: Splitter 2x1 at grid position (0, 1) - spans 2 columns
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 4: Splitter 3x1 at grid position (2, 1) - spans 3 columns
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 5: Splitter 4x1 at grid position (5, 1) - spans 4 columns
    layout.add_texture(grid_rect(0, 2, 4, 1));

    // Index 6: Combiner 2x1 at grid position (0, 2) - spans 2 columns
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 7: Combiner 3x1 at grid position (2, 2) - spans 3 columns
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 8: Combiner 4x1 at grid position (5, 2) - spans 4 columns
    layout.add_texture(grid_rect(0, 2, 4, 1));

    // Index 9: Delinker 2x1 at grid position (0, 3) - spans 2 columns
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 10: Delinker 3x1 at grid position (2, 3) - spans 3 columns
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 11: Delinker 4x1 at grid position (5, 3) - spans 4 columns
    layout.add_texture(grid_rect(0, 2, 4, 1));

    // Index 12: Trunker 2x1 at grid position (0, 4) - spans 2 columns
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 13: Trunker 3x1 at grid position (2, 4) - spans 3 columns
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 14: Trunker 4x1 at grid position (5, 4) - spans 4 columns
    layout.add_texture(grid_rect(0, 2, 4, 1));

    layout
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
        2,                  // The number of columns
        2,                  // The number of rows
        None,               // Optional padding
        None,               // Optional offset
    );

    let layout_handle = texture_atlas_layouts.add(layout);

    // Load buildings spritesheet with variable-sized sprites
    let buildings_texture_handle = asset_server.load("spritesheet.png");
    let buildings_layout = create_buildings_layout();
    let buildings_layout_handle = texture_atlas_layouts.add(buildings_layout);

    let game_assets = GameAssets {
        icons_texture: texture_handle,
        icons_layout: layout_handle,
        transparent_icons_texture: transparent_texture_handle,
        buildings_texture: buildings_texture_handle,
        buildings_layout: buildings_layout_handle,
    };
    commands.insert_resource(game_assets);
}
