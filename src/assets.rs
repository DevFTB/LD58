use bevy::math::URect;
use bevy::prelude::*;
use std::collections::HashMap;
use crate::factions::Faction;

/// Machine types for buildings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MachineType {
    Collector,
    Aggregator,
    Splitter,
    Combiner,
    Delinker,
    Trunker,
}

/// Machine variant for buildings that come in different sizes
/// For buildings like Splitter, Combiner, etc. that have 2x1, 3x1, 4x1 variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MachineVariant {
    Single,    // For 1x1 buildings like Collector, Aggregator
    Size2,     // For 2x1 buildings
    Size3,     // For 3x1 buildings
    Size4,     // For 4x1 buildings
}

/// Combined key for looking up machine sprites
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MachineKey {
    pub machine_type: MachineType,
    pub variant: MachineVariant,
}

impl MachineKey {
    pub fn new(machine_type: MachineType, variant: MachineVariant) -> Self {
        Self { machine_type, variant }
    }
    
    /// Convenience constructor for single-size buildings
    pub fn single(machine_type: MachineType) -> Self {
        Self { machine_type, variant: MachineVariant::Single }
    }
}

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
    pub machines_texture: Handle<Image>,
    pub machines_layout: Handle<TextureAtlasLayout>,
    pub machines: HashMap<MachineKey, usize>,
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
    
    /// Get texture atlas index for a machine sprite
    /// Returns the atlas index for the specified machine type and variant
    pub fn machine_sprite(&self, machine_type: MachineType, variant: MachineVariant) -> Option<usize> {
        let key = MachineKey::new(machine_type, variant);
        self.machines.get(&key).copied()
    }
    
    /// Get texture atlas index for a single-size machine (convenience method)
    pub fn machine_sprite_single(&self, machine_type: MachineType) -> Option<usize> {
        self.machine_sprite(machine_type, MachineVariant::Single)
    }
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
    let min_x = col * 32;
    let min_y = row * 32;
    let max_x = min_x + (width * 32);
    let max_y = min_y + (height * 32);

    URect::new(min_x, min_y, max_x, max_y)
}

/// Creates the texture atlas layout for buildings with variable sizes
/// Add your sprites here using grid_rect(column, row, width, height)
fn create_buildings_layout() -> TextureAtlasLayout {
    let sheet_size = UVec2::new(12 * 32, 8 * 32);
    let mut layout = TextureAtlasLayout::new_empty(sheet_size);

    // Index 0: Collector (1x1)
    layout.add_texture(grid_rect(0, 0, 1, 1));

    // Index 1: Aggregator (1x1)
    layout.add_texture(grid_rect(4, 0, 1, 1));

    // Index 2: Splitter 2x1 
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 3: Splitter 3x1 
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 4: Splitter 4x1 
    layout.add_texture(grid_rect(0, 2, 4, 1));

    // Index 5: Combiner 2x1
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 6: Combiner 3x1
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 7: Combiner 4x1
    layout.add_texture(grid_rect(0, 2, 4, 1));

    // Index 8: Delinker 2x1
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 9: Delinker 3x1
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 10: Delinker 4x1
    layout.add_texture(grid_rect(0, 2, 4, 1));

    // Index 11: Trunker 2x1
    layout.add_texture(grid_rect(0, 1, 2, 1));

    // Index 12: Trunker 3x1
    layout.add_texture(grid_rect(0, 3, 3, 1));

    // Index 13: Trunker 4x1
    layout.add_texture(grid_rect(0, 2, 4, 1));

    layout
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

    let machines_texture = asset_server.load::<Image>("machines.png");
    let matchines_layout = create_buildings_layout();
    let machines_layout_handle = texture_atlas_layouts.add(matchines_layout);

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

    // Initialize machine sprite mappings
    let mut machines = HashMap::new();
    
    // Single-size buildings
    machines.insert(MachineKey::single(MachineType::Collector), 0);
    machines.insert(MachineKey::single(MachineType::Aggregator), 1);
    
    // Splitter variants (2x1, 3x1, 4x1)
    machines.insert(MachineKey::new(MachineType::Splitter, MachineVariant::Size2), 2);
    machines.insert(MachineKey::new(MachineType::Splitter, MachineVariant::Size3), 3);
    machines.insert(MachineKey::new(MachineType::Splitter, MachineVariant::Size4), 4);
    
    // Combiner variants (2x1, 3x1, 4x1)
    machines.insert(MachineKey::new(MachineType::Combiner, MachineVariant::Size2), 5);
    machines.insert(MachineKey::new(MachineType::Combiner, MachineVariant::Size3), 6);
    machines.insert(MachineKey::new(MachineType::Combiner, MachineVariant::Size4), 7);
    
    // Delinker variants (2x1, 3x1, 4x1)
    machines.insert(MachineKey::new(MachineType::Delinker, MachineVariant::Size2), 8);
    machines.insert(MachineKey::new(MachineType::Delinker, MachineVariant::Size3), 9);
    machines.insert(MachineKey::new(MachineType::Delinker, MachineVariant::Size4), 10);
    
    // Trunker variants (2x1, 3x1, 4x1)
    machines.insert(MachineKey::new(MachineType::Trunker, MachineVariant::Size2), 11);
    machines.insert(MachineKey::new(MachineType::Trunker, MachineVariant::Size3), 12);
    machines.insert(MachineKey::new(MachineType::Trunker, MachineVariant::Size4), 13);

    let game_assets = GameAssets {
        small_sprites_texture: small_sprites_handle,
        small_sprites_layout: small_sprites_layout_handle,
        faction_colors,
        faction_icons,
        utility_icons,
        machines_layout: machines_layout_handle,
        machines_texture,
        machines,
    };
    commands.insert_resource(game_assets);
}
