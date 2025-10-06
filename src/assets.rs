use bevy::prelude::*;
use std::collections::HashMap;
use crate::factions::Faction;

/// Utility icon indices for common UI sprites
#[derive(Debug, Clone)]
pub struct UtilityIcons {
    pub arrow_up: usize,
    pub arrow_double_up: usize,
    pub arrow_down: usize,
    pub arrow_double_down: usize,
    pub money: usize,
}

// The main resource now holds handles for textures, colors, and icons
#[derive(Resource)]
pub struct GameAssets {
    pub small_sprites_texture: Handle<Image>,
    pub small_sprites_layout: Handle<TextureAtlasLayout>,
    pub faction_colors: HashMap<Faction, Color>,
    pub faction_icons: HashMap<Faction, usize>,
    pub utility_icons: UtilityIcons,
}

impl GameAssets {
    /// Get color for a faction
    pub fn faction_color(&self, faction: Faction) -> Color {
        *self.faction_colors.get(&faction).unwrap_or(&Color::WHITE)
    }

    /// Get texture atlas index for a faction icon
    /// All factions are guaranteed to have icons
    pub fn faction_icon(&self, faction: Faction) -> usize {
        *self.faction_icons.get(&faction).expect("All factions must have icons")
    }
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_assets);
    }
}

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let small_sprites_handle = asset_server.load::<Image>("small_sprites.png");

    let small_sprites_layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), // The size of each sprite
        6,                    // The number of columns
        6,                    // The number of rows
        None,                 // Optional padding
        None,                 // Optional offset
    );

    let small_sprites_layout_handle = texture_atlas_layouts.add(small_sprites_layout);

    // Initialize faction colors
    let mut faction_colors = HashMap::new();
    faction_colors.insert(Faction::Academia, Color::srgb(0.2, 0.8, 1.0));    // Cyan
    faction_colors.insert(Faction::Corporate, Color::srgb(0.9, 0.9, 0.3));   // Yellow
    faction_colors.insert(Faction::Government, Color::srgb(0.3, 1.0, 0.3));  // Green
    faction_colors.insert(Faction::Criminal, Color::srgb(1.0, 0.3, 0.3));    // Red

    // Initialize faction icons (map to texture atlas indices)
    let mut faction_icons = HashMap::new();
    faction_icons.insert(Faction::Academia, 24);
    faction_icons.insert(Faction::Corporate, 18);
    faction_icons.insert(Faction::Government, 12);
    faction_icons.insert(Faction::Criminal, 6);

    // Initialize utility icons
    let utility_icons = UtilityIcons {
        arrow_up: 30,           // Map these to actual indices in your sprite sheet
        arrow_double_up: 31,
        arrow_down: 33,
        arrow_double_down: 32,
        money: 1, //Temporary index for money icon
    };

    let game_assets = GameAssets {
        small_sprites_texture: small_sprites_handle,
        small_sprites_layout: small_sprites_layout_handle,
        faction_colors,
        faction_icons,
        utility_icons,
    };
    commands.insert_resource(game_assets);
}

