use bevy::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use crate::factions::Faction;
use crate::factory::logical::BasicDataType;

/// Identifies which texture atlas to use for a building sprite
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum AtlasId {
    SmallSprites,
    LargeSprites,
    Buildings1x1,
    Buildings2x1,
    Buildings3x1,
    Buildings4x1, 
    SourceBackgrounds,
    Wires,
}

/// Size variants for icons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    Small,  // 16x16
    Large,  // 32x32
}


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

impl MachineVariant {
    /// Get the AtlasId for this variant
    pub fn atlas_id(&self) -> AtlasId {
        match self {
            MachineVariant::Single => AtlasId::Buildings1x1,
            MachineVariant::Size2 => AtlasId::Buildings2x1,
            MachineVariant::Size3 => AtlasId::Buildings3x1,
            MachineVariant::Size4 => AtlasId::Buildings4x1,
        }
    }
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
    pub data_sprites_texture: Handle<Image>,
    pub data_sprites_layout: Handle<TextureAtlasLayout>,
    pub faction_colors: HashMap<Faction, Color>,
    pub faction_icons_small: HashMap<Faction, usize>,
    pub faction_icons_large: HashMap<Faction, usize>,
    pub utility_icons: UtilityIcons,
    
    // Building texture atlases (one per size)
    pub buildings_1x1_texture: Handle<Image>,
    pub buildings_1x1_layout: Handle<TextureAtlasLayout>,
    pub buildings_2x1_texture: Handle<Image>,
    pub buildings_2x1_layout: Handle<TextureAtlasLayout>,
    pub buildings_3x1_texture: Handle<Image>,
    pub buildings_3x1_layout: Handle<TextureAtlasLayout>,
    pub buildings_4x1_texture: Handle<Image>,
    pub buildings_4x1_layout: Handle<TextureAtlasLayout>,
    
    // Source backgrounds atlas
    pub source_backgrounds_texture: Handle<Image>,
    pub source_backgrounds_layout: Handle<TextureAtlasLayout>,
    
    // Wires atlas (different orientations)
    pub wires_texture: Handle<Image>,
    pub wires_layout: Handle<TextureAtlasLayout>,
    
    // Machine sprite index mappings (atlas is derived from variant)
    pub machines: HashMap<MachineKey, usize>,
    
    // Data type icon mappings for source visualization
    pub data_type_icons_small: HashMap<BasicDataType, usize>,
    pub data_type_icons_large: HashMap<BasicDataType, usize>,
    
    pub font: Handle<Font>,
}

impl GameAssets {
    /// Get color for a faction
    pub fn faction_color(&self, faction: Faction) -> Color {
        *self.faction_colors.get(&faction).unwrap_or(&Color::WHITE)
    }

    /// Create a TextFont with the game's custom font
    /// Use this instead of `TextFont::default()` to ensure consistent font usage
    pub fn text_font(&self, font_size: f32) -> TextFont {
        TextFont {
            font: self.font.clone(),
            font_size,
            ..default()
        }
    }
    
    /// Get texture and layout handles for a specific atlas
    pub fn get_atlas(&self, atlas_id: AtlasId) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
        match atlas_id {
            AtlasId::SmallSprites => (self.small_sprites_texture.clone(), self.small_sprites_layout.clone()),
            AtlasId::LargeSprites => (self.data_sprites_texture.clone(), self.data_sprites_layout.clone()),
            AtlasId::Buildings1x1 => (self.buildings_1x1_texture.clone(), self.buildings_1x1_layout.clone()),
            AtlasId::Buildings2x1 => (self.buildings_2x1_texture.clone(), self.buildings_2x1_layout.clone()),
            AtlasId::Buildings3x1 => (self.buildings_3x1_texture.clone(), self.buildings_3x1_layout.clone()),
            AtlasId::Buildings4x1 => (self.buildings_4x1_texture.clone(), self.buildings_4x1_layout.clone()),
            AtlasId::SourceBackgrounds => (self.source_backgrounds_texture.clone(), self.source_backgrounds_layout.clone()),
            AtlasId::Wires => (self.wires_texture.clone(), self.wires_layout.clone()),
        }
    }
    
    /// Get atlas ID and sprite index for a machine sprite
    /// Returns (AtlasId, sprite_index) - AtlasId is derived from the variant
    pub fn machine_sprite(&self, machine_type: MachineType, variant: MachineVariant) -> Option<(AtlasId, usize)> {
        let key = MachineKey::new(machine_type, variant);
        self.machines.get(&key).map(|&index| (variant.atlas_id(), index))
    }
    
    /// Get atlas ID and sprite index for a single-size machine (convenience method)
    pub fn machine_sprite_single(&self, machine_type: MachineType) -> Option<(AtlasId, usize)> {
        self.machine_sprite(machine_type, MachineVariant::Single)
    }

    /// Get atlas ID and sprite index for a faction icon
    /// Returns (AtlasId, sprite_index) - AtlasId is derived from the size
    pub fn faction_icon(&self, faction: Faction, size: IconSize) -> Option<(AtlasId, usize)> {
        match size {
            IconSize::Small => self.faction_icons_small.get(&faction).map(|&index| (AtlasId::SmallSprites, index)),
            IconSize::Large => self.faction_icons_large.get(&faction).map(|&index| (AtlasId::LargeSprites, index)),
        }
    }

    /// Get atlas ID and sprite index for a data type icon
    /// Returns (AtlasId, sprite_index) - AtlasId is derived from the size
    pub fn data_type_icon(&self, data_type: BasicDataType, size: IconSize) -> Option<(AtlasId, usize)> {
        match size {
            IconSize::Small => self.data_type_icons_small.get(&data_type).map(|&index| (AtlasId::SmallSprites, index)),
            IconSize::Large => self.data_type_icons_large.get(&data_type).map(|&index| (AtlasId::LargeSprites, index)),
        }
    }

    /// Get background sprite index for a faction source
    /// Faction sources have their own background sprites (indices 0-3)
    pub fn faction_background_index(&self, faction: Faction) -> usize {
        match faction {
            Faction::Academia => 7,
            Faction::Corporate => 5,
            Faction::Government => 3,
            Faction::Criminal => 1,
        }
    }

    /// Get background sprite index for a data type source
    /// Data type sources use backgrounds based on their primary data type (indices 4-7)
    pub fn datatype_background_index(&self, data_type: BasicDataType) -> usize {
        match data_type {
            BasicDataType::Biometric => 0,
            BasicDataType::Economic => 2,
            BasicDataType::Behavioural => 4,
            BasicDataType::Telemetry => 6,
        }
    }
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_assets)
           .add_systems(Startup, play_background_audio);
    }
}

pub fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load small sprites atlas (UI icons, faction icons, etc.)
    let small_sprites_handle = asset_server.load::<Image>("small_sprites.png");
    let small_sprites_layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), // The size of each sprite
        6,                    // The number of columns
        6,                    // The number of rows
        None,                 // Optional padding
        None,                 // Optional offset
    );
    let small_sprites_layout_handle = texture_atlas_layouts.add(small_sprites_layout);

    // Load large sprites atlas (32x32 icons for source visuals)
    let data_sprites_handle = asset_server.load::<Image>("datatypes/Basic data.png");
    let data_sprites_layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32), // The size of each sprite
        5,                    // The number of columns
        4,                    // The number of rows
        None,                 // Optional padding
        None,                 // Optional offset
    );
    let large_sprites_layout_handle = texture_atlas_layouts.add(data_sprites_layout);

    // Load building texture atlases (separate file for each size)
    let buildings_1x1_texture = asset_server.load::<Image>("buildings/1x1.png");
    let buildings_1x1_layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),  // 1x1 sprites are 32x32
        4,                    // columns - adjust based on your sprite sheet
        4,                    // rows - adjust based on your sprite sheet
        None,
        None,
    );
    let buildings_1x1_layout_handle = texture_atlas_layouts.add(buildings_1x1_layout);

    let buildings_2x1_texture = asset_server.load::<Image>("buildings/2x1.png");
    let buildings_2x1_layout = TextureAtlasLayout::from_grid(
        UVec2::new(64, 32),  // 2x1 sprites are 64x32
        1,                    // columns - adjust based on your sprite sheet
        4,                    // rows - adjust based on your sprite sheet
        None,
        None,
    );
    let buildings_2x1_layout_handle = texture_atlas_layouts.add(buildings_2x1_layout);

    let buildings_3x1_texture = asset_server.load::<Image>("buildings/3x1 machines.png");
    let buildings_3x1_layout = TextureAtlasLayout::from_grid(
        UVec2::new(96, 32),  // 3x1 sprites are 96x32
        1,                    // columns - adjust based on your sprite sheet
        4,                    // rows - adjust based on your sprite sheet
        None,
        None,
    );
    let buildings_3x1_layout_handle = texture_atlas_layouts.add(buildings_3x1_layout);

    let buildings_4x1_texture = asset_server.load::<Image>("buildings/4x1.png");
    let buildings_4x1_layout = TextureAtlasLayout::from_grid(
        UVec2::new(128, 32), // 4x1 sprites are 128x32
        1,                    // columns - adjust based on your sprite sheet
        4,                    // rows - adjust based on your sprite sheet
        None,
        None,
    );
    let buildings_4x1_layout_handle = texture_atlas_layouts.add(buildings_4x1_layout);

    // Load source backgrounds atlas
    let source_backgrounds_texture = asset_server.load::<Image>("buildings/source_backgrounds.png");
    let source_backgrounds_layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),  // Adjust size based on your sprite sheet
        2,                    // columns - adjust based on your sprite sheet
        4,                    // rows - adjust based on your sprite sheet
        None,
        None,
    );
    let source_backgrounds_layout_handle = texture_atlas_layouts.add(source_backgrounds_layout);

    // Load wires atlas (different orientations)
    let wires_texture = asset_server.load::<Image>("wires.png");
    let wires_layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),  // Adjust size based on your sprite sheet
        2,                    // columns - adjust based on your sprite sheet
        4,                    // rows - adjust based on your sprite sheet
        None,
        None,
    );
    let wires_layout_handle = texture_atlas_layouts.add(wires_layout);

    // Initialize faction colors
    let mut faction_colors = HashMap::new();
    faction_colors.insert(Faction::Academia, Color::srgb(0.2, 0.8, 1.0));    // Cyan
    faction_colors.insert(Faction::Corporate, Color::srgb(0.9, 0.9, 0.3));   // Yellow
    faction_colors.insert(Faction::Government, Color::srgb(0.3, 1.0, 0.3));  // Green
    faction_colors.insert(Faction::Criminal, Color::srgb(1.0, 0.3, 0.3));    // Red

    // Initialize faction icons - small (16x16) in small_sprites atlas
    let mut faction_icons_small = HashMap::new();
    faction_icons_small.insert(Faction::Academia, 24);
    faction_icons_small.insert(Faction::Corporate, 18);
    faction_icons_small.insert(Faction::Government, 12);
    faction_icons_small.insert(Faction::Criminal, 6);

    // Initialize faction icons - large (32x32) in large_sprites atlas
    let mut faction_icons_large = HashMap::new();
    faction_icons_large.insert(Faction::Academia, 24);
    faction_icons_large.insert(Faction::Corporate, 18);
    faction_icons_large.insert(Faction::Government, 12);
    faction_icons_large.insert(Faction::Criminal, 6);

    // Initialize utility icons (in small_sprites atlas)
    let utility_icons = UtilityIcons {
        arrow_up: 30,
        arrow_double_up: 31,
        arrow_down: 33,
        arrow_double_down: 32,
        money: 1,
    };

    // Initialize machine sprite mappings: MachineKey -> sprite_index
    // AtlasId is automatically derived from the MachineVariant
    let mut machines = HashMap::new();
    
    // 1x1 buildings (Collector at index 0, Aggregator at index 1)
    machines.insert(MachineKey::single(MachineType::Collector), 1);
    machines.insert(MachineKey::single(MachineType::Aggregator),0);
    
    // 2x1 buildings (Splitter, Combiner, Delinker, Trunker)
    machines.insert(MachineKey::new(MachineType::Splitter, MachineVariant::Size2),1);
    machines.insert(MachineKey::new(MachineType::Combiner, MachineVariant::Size2), 0);
    machines.insert(MachineKey::new(MachineType::Delinker, MachineVariant::Size2), 3);
    machines.insert(MachineKey::new(MachineType::Trunker, MachineVariant::Size2), 2);
    
    // 3x1 buildings (Splitter, Combiner, Delinker, Trunker)
    machines.insert(MachineKey::new(MachineType::Splitter, MachineVariant::Size3), 1);
    machines.insert(MachineKey::new(MachineType::Combiner, MachineVariant::Size3), 3);
    machines.insert(MachineKey::new(MachineType::Delinker, MachineVariant::Size3), 3);
    machines.insert(MachineKey::new(MachineType::Trunker, MachineVariant::Size3), 2);
    
    // 4x1 buildings (Splitter, Combiner, Delinker, Trunker)
    machines.insert(MachineKey::new(MachineType::Splitter, MachineVariant::Size4), 1);
    machines.insert(MachineKey::new(MachineType::Combiner, MachineVariant::Size4), 0);
    machines.insert(MachineKey::new(MachineType::Delinker, MachineVariant::Size4), 3);
    machines.insert(MachineKey::new(MachineType::Trunker, MachineVariant::Size4), 2);

    // Load font
    let font_handle = asset_server.load::<Font>("Fonts/Bitcount_Grid_Double_Ink/BitcountGridDoubleInk-VariableFont_CRSV,ELSH,ELXP,SZP1,SZP2,XPN1,XPN2,YPN1,YPN2,slnt,wght.ttf");

    // Map data types to sprite indices - small (16x16) in small_sprites atlas
    let mut data_type_icons_small = HashMap::new();
    data_type_icons_small.insert(BasicDataType::Biometric, 0);   // A - First icon in atlas
    data_type_icons_small.insert(BasicDataType::Economic, 1);    // B - Second icon
    data_type_icons_small.insert(BasicDataType::Behavioural, 2); // C - Third icon
    data_type_icons_small.insert(BasicDataType::Telemetry, 3);   // D - Fourth icon

    // Map data types to sprite indices - large (32x32) in large_sprites atlas
    let mut data_type_icons_large = HashMap::new();
    data_type_icons_large.insert(BasicDataType::Biometric, 0);   // A - First icon in atlas
    data_type_icons_large.insert(BasicDataType::Economic, 5);    // B - Second icon
    data_type_icons_large.insert(BasicDataType::Behavioural, 10); // C - Third icon
    data_type_icons_large.insert(BasicDataType::Telemetry, 15);   // D - Fourth icon

    let game_assets = GameAssets {
        small_sprites_texture: small_sprites_handle,
        small_sprites_layout: small_sprites_layout_handle,
        data_sprites_texture: data_sprites_handle,
        data_sprites_layout: large_sprites_layout_handle,
        faction_colors,
        faction_icons_small,
        faction_icons_large,
        utility_icons,
        buildings_1x1_texture,
        buildings_1x1_layout: buildings_1x1_layout_handle,
        buildings_2x1_texture,
        buildings_2x1_layout: buildings_2x1_layout_handle,
        buildings_3x1_texture,
        buildings_3x1_layout: buildings_3x1_layout_handle,
        buildings_4x1_texture,
        buildings_4x1_layout: buildings_4x1_layout_handle,
        source_backgrounds_texture,
        source_backgrounds_layout: source_backgrounds_layout_handle,
        wires_texture,
        wires_layout: wires_layout_handle,
        machines,
        data_type_icons_small,
        data_type_icons_large,
        font: font_handle.clone(),
    };

    commands.insert_resource(game_assets);
}

/// Play looped background audio
pub fn play_background_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Try to play looped background audio
    // Note: Bevy supports OGG Vorbis (.ogg) and FLAC (.flac) by default
    // WAV files need to be in a specific format (PCM) to work
    let audio_handle: Handle<AudioSource> = asset_server.load("data_collection.ogg");
    commands.spawn((
        AudioPlayer::new(audio_handle),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.05)),
    ));
}
