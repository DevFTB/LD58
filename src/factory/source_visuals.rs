use bevy::prelude::*;
use bevy::platform::collections::HashSet; // Use Bevy's HashSet
use bevy::render::render_resource::{FilterMode, SamplerDescriptor};
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use crate::factory::logical::{DataAttribute, BasicDataType};
use crate::factory::buildings::source::SourceBuilding;
use crate::assets::{GameAssets, AtlasId};
use crate::grid::{GridPosition, GridAtlasSprite};
use crate::factions::Faction;

/// Component that marks a source building's background sprite
#[derive(Component)]
pub struct SourceBackground;

/// Component that marks a data type icon overlay on a source
#[derive(Component)]
pub struct DataTypeIcon {
    pub data_type: crate::factory::logical::BasicDataType,
    pub parent_source: Entity,
}

/// Component that marks augmentation visual effects
#[derive(Component)]
pub struct AugmentationEffect {
    pub attribute: DataAttribute,
    pub parent_icon: Entity,
}

/// Component for golden bloom glow effect (for augmented data)
#[derive(Component)]
pub struct GoldenGlowEffect {
    pub parent_icon: Entity,
    pub intensity: f32,
}

/// Component that marks the glow sprite entity (the actual visual glow behind the icon)
#[derive(Component)]
pub struct GlowSprite {
    pub parent_icon: Entity,
}

/// Component to track orbital motion parameters for glow particles
#[derive(Component)]
pub struct GlowParticleOrbit {
    pub base_position: Vec3,
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    pub initial_angle: f32,
}

/// Component for scanning flash effect (for identified data)
#[derive(Component)]
pub struct ScanningFlashEffect {
    pub parent_icon: Entity,
    pub timer: f32,
    pub flash_interval: f32,
}

/// Component for augmented data indicator sprite with pulse animation
#[derive(Component)]
pub struct AugmentedIndicator {
    pub parent_icon: Entity,
    pub base_scale: f32,
    pub time_offset: f32, // Random offset to desync pulse animations
}

/// Component for floating animation on icons
#[derive(Component)]
pub struct FloatingAnimation {
    pub base_y: f32,
    pub time_offset: f32, // Random offset to desync animations
}

/// System to spawn background sprites for source buildings
pub fn spawn_source_backgrounds(
    mut commands: Commands,
    query: Query<(Entity, &SourceBuilding, &GridPosition, Option<&Faction>), Added<SourceBuilding>>,
    game_assets: Res<GameAssets>,
    grid: Res<crate::grid::Grid>,
    asset_server: Res<AssetServer>,
) {
    for (_entity, source, grid_pos, faction) in query.iter() {
        // Determine background sprite index based on faction or data type
        let background_index = if let Some(faction) = faction {
            // Faction sources: use faction-specific background (indices 0-3)
            game_assets.faction_background_index(*faction)
        } else {
            // Non-faction sources: use background based on primary data type (indices 4-7)
            // Get the first data type from the source's dataset
            source.shape.contents.keys().next()
                .map(|data_type| game_assets.datatype_background_index(*data_type))
                .unwrap_or(4) // Fallback to index 4 if no data types (shouldn't happen)
        };

        // Get the texture and layout for the background atlas
        let (texture, layout) = game_assets.get_atlas(AtlasId::SourceBackgrounds);
        
        // Calculate sprite size in pixels
        let sprite_width = source.size.x as f32 * grid.scale;
        let sprite_height = source.size.y as f32 * grid.scale;

        // Calculate the proper world position using the grid system
        let position = grid.calculate_building_sprite_position(
            grid_pos,
            source.size.x,
            source.size.y,
            crate::grid::Orientation::default(),
        );

        // Spawn background sprite at the calculated grid position, behind source tiles (z = -1.0)
        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::new(sprite_width, sprite_height)),
                image: texture,
                texture_atlas: Some(TextureAtlas {
                    layout,
                    index: background_index,
                }),
                ..Default::default()
            },
            Transform::from_translation(position.extend(-1.0)),
            SourceBackground,
            Visibility::default(),
        ));
    }
}

/// System to spawn/update data type icon overlays based on the dataset
pub fn update_source_data_icons(
    mut commands: Commands,
    sources_query: Query<(Entity, &SourceBuilding, &GridPosition), Changed<SourceBuilding>>,
    existing_icons: Query<(Entity, &DataTypeIcon)>,
    game_assets: Res<GameAssets>,
    grid: Res<crate::grid::Grid>,
    asset_server: Res<AssetServer>,
) {
    for (source_entity, source, grid_pos) in sources_query.iter() {
        // Remove existing icons for this source
        for (icon_entity, icon) in existing_icons.iter() {
            if icon.parent_source == source_entity {
                commands.entity(icon_entity).despawn();
            }
        }

        // Get data types from the source's dataset and randomize order
        let mut data_types: Vec<_> = source.shape.contents.keys().cloned().collect();
        // Use entity index as seed for consistent but randomized ordering per source
        let seed = source_entity.index() as usize;
        // Simple shuffle based on entity index
        if data_types.len() > 1 {
            data_types.sort_by_key(|dt| {
                // Create a pseudo-random value based on data type and entity
                let type_hash = match dt {
                    crate::factory::logical::BasicDataType::Biometric => 1,
                    crate::factory::logical::BasicDataType::Economic => 2,
                    crate::factory::logical::BasicDataType::Behavioural => 3,
                    crate::factory::logical::BasicDataType::Telemetry => 4,
                };
                (type_hash * 7 + seed) % 100
            });
        }
        
        // Calculate base position for the source
        let base_position = grid.calculate_building_sprite_position(
            grid_pos,
            source.size.x,
            source.size.y,
            crate::grid::Orientation::default(),
        );
        
        // Spawn new icons for each data type
        let num_icons = data_types.len();
        for (index, data_type) in data_types.iter().enumerate() {
            if let Some(&sprite_index) = game_assets.data_type_icons_large.get(data_type) {
                // Get the texture and layout for large sprites (32x32)
                let (texture, layout) = game_assets.get_atlas(AtlasId::LargeSprites);
                
                // Calculate positioning for clustered icons
                let icon_size = 32.0; // Large sprites are 32x32
                
                // Calculate offset based on layout pattern
                // Scale up single icons to be more prominent
                let icon_display_size = if num_icons == 1 { 48.0 } else { icon_size };
                
                let (offset_x, offset_y) = match num_icons {
                    1 => {
                        // Single icon at center
                        (0.0, 0.0)
                    }
                    2 => {
                        // Two icons side by side with slight overlap
                        let spacing = icon_size * 0.6;
                        let x = if index == 0 { -spacing / 2.0 } else { spacing / 2.0 };
                        (x, 0.0)
                    }
                    3 => {
                        // Triangular arrangement (3-way Venn diagram style)
                        // Overlap amount: icons overlap by ~30% for Venn diagram effect
                        let overlap = icon_size * 0.7; // 70% of icon size = 30% overlap
                        match index {
                            0 => (0.0, overlap * 0.5),           // Top center
                            1 => (-overlap * 0.5, -overlap * 0.3), // Bottom left
                            2 => (overlap * 0.5, -overlap * 0.3),  // Bottom right
                            _ => (0.0, 0.0)
                        }
                    }
                    _ => {
                        // 4+ icons: horizontal line with tight spacing
                        let spacing = icon_size * 0.4;
                        let total_width = (num_icons - 1) as f32 * spacing;
                        let x = (index as f32 * spacing) - (total_width / 2.0);
                        (x, 0.0)
                    }
                };
                
                // Spawn icon as a regular sprite at the source's position with offset
                // For triangular layout (3 icons), put the top icon (index 0) behind the others
                let z_order = if num_icons == 3 && index == 0 {
                    0.9 // Top icon slightly behind
                } else {
                    1.0 // Other icons above
                };
                
                let icon_transform = Transform::from_translation(Vec3::new(
                    base_position.x + offset_x,
                    base_position.y + offset_y,
                    z_order,
                ));
                
                // Calculate time offset for floating animation desync
                let time_offset = (index as f32) * 1.5 + (source_entity.index() as f32 * 0.1);
                
                let icon = commands
                    .spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(icon_display_size, icon_display_size)),
                            image: texture,
                            texture_atlas: Some(TextureAtlas {
                                layout,
                                index: sprite_index,
                            }),
                            ..Default::default()
                        },
                        icon_transform,
                        DataTypeIcon {
                            data_type: data_type.clone(),
                            parent_source: source_entity,
                        },
                        FloatingAnimation {
                            base_y: icon_transform.translation.y,
                            time_offset,
                        },
                        Visibility::default(),
                    ))
                    .id();

                // Check for augmentations on this data type
                if let Some(attributes) = source.shape.contents.get(data_type) {
                    spawn_augmentation_effects(&mut commands, icon, &icon_transform, attributes, &asset_server);
                }
            }
        }
    }
}

/// Helper function to spawn visual effects for augmented data
fn spawn_augmentation_effects(
    commands: &mut Commands,
    icon_entity: Entity,
    icon_transform: &Transform,
    attributes: &HashSet<DataAttribute>,
    asset_server: &AssetServer,
) {
    // Check if data is augmented (has Aggregated or Cleaned attributes)
    let has_augmentation = is_data_augmented(attributes);
    
    // Check if data is identified (NOT deidentified)
    let is_identified = is_data_identified(attributes);
    
    // Add augmented indicator sprite for augmented data
    if has_augmentation {
        let augmented_texture = asset_server.load("augmented.png");
        
        // Position at top-right, slightly above the icon
        let indicator_position = icon_transform.translation + Vec3::new(10.0, 14.0, 0.2);
        
        // Add random offset for pulse animation desync
        let pulse_offset = (icon_entity.index() as f32 * 0.3) % 2.0;
        
        commands.spawn((
            Sprite {
                image: augmented_texture,
                custom_size: Some(Vec2::new(12.0, 12.0)), // Small indicator
                ..Default::default()
            },
            Transform::from_translation(indicator_position)
                .with_scale(Vec3::splat(1.0)),
            AugmentedIndicator {
                parent_icon: icon_entity,
                base_scale: 1.0,
                time_offset: pulse_offset,
            },
        ));
    }
    
    // Add scanning flash effect for identified data
    if is_identified {
        // Add random offset to flash timing for desync
        let flash_offset = (icon_entity.index() as f32 * 0.25) % 3.0;
        commands.entity(icon_entity).insert(ScanningFlashEffect {
            parent_icon: icon_entity,
            timer: flash_offset,
            flash_interval: 3.0,
        });
    }
}



/// System to animate scanning flash effect for identified data
pub fn animate_scanning_flash(
    time: Res<Time>,
    mut flash_query: Query<(&mut ScanningFlashEffect, &mut Sprite)>,
) {
    let delta = time.delta_secs();
    
    for (mut flash, mut sprite) in flash_query.iter_mut() {
        flash.timer += delta;
        
        // Create a scanning flash effect at intervals
        if flash.timer >= flash.flash_interval {
            flash.timer = 0.0;
        }
        
        // Flash lasts for 0.8 seconds (much slower), creating a scanning effect
        let flash_duration = 0.8;
        if flash.timer < flash_duration {
            // Overall progress of the flash (0 to 1)
            let progress = flash.timer / flash_duration;
            
            // Create a double-peak scanning effect that simulates a scan line passing through
            // The scan peaks at 25% and 75% of the duration, simulating top and bottom of scan
            let scan_wave = ((progress * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5).powf(2.0);
            
            // Add a quick bright pulse at the start (scan initiation)
            let initial_pulse = if progress < 0.15 {
                (1.0 - (progress / 0.15)).powf(2.0)
            } else {
                0.0
            };
            
            // Combine the scanning wave with initial pulse
            let flash_intensity = (scan_wave * 0.7 + initial_pulse * 0.8).min(1.0);
            
            // Create very bright cyan/white scanning laser effect
            let brightness = 1.0 + flash_intensity * 1.5; // 1.0 to 2.5 (very overbright)
            let cyan_tint = 0.8 + flash_intensity * 0.2; // Slight cyan tint (0.8 to 1.0)
            sprite.color = Color::srgb(brightness, brightness, brightness * cyan_tint);
        } else {
            // Reset to normal color when not flashing
            sprite.color = Color::WHITE;
        }
    }
}

/// System to animate floating icons (slow bounce up and down)
pub fn animate_floating_icons(
    time: Res<Time>,
    mut icon_query: Query<(&FloatingAnimation, &mut Transform), With<DataTypeIcon>>,
) {
    let t = time.elapsed_secs();
    
    for (float_anim, mut transform) in icon_query.iter_mut() {
        // Apply time offset for desync
        let desynced_t = t + float_anim.time_offset;
        
        // Very slow sine wave for floating (period of ~4 seconds)
        let float_offset = (desynced_t * 0.5).sin() * 3.0; // ±3 pixels
        
        // Update Y position
        transform.translation.y = float_anim.base_y + float_offset;
    }
}

/// System to animate pulse effect on augmented indicators
pub fn animate_augmented_pulse(
    time: Res<Time>,
    mut indicator_query: Query<(&AugmentedIndicator, &mut Transform)>,
) {
    for (indicator, mut transform) in indicator_query.iter_mut() {
        // Create desynced time value
        let desynced_t = time.elapsed_secs() + indicator.time_offset;
        
        // Subtle pulse: scale oscillates between 0.95 and 1.05 (10% range)
        // Using sine wave with period of ~2 seconds (frequency of 1.0)
        let pulse_factor = 1.0 + (desynced_t * std::f32::consts::PI).sin() * 0.05;
        
        let new_scale = indicator.base_scale * pulse_factor;
        transform.scale = Vec3::splat(new_scale);
    }
}

/// Public helper function to check if data has augmentation attributes
/// This can be called from anywhere in the game to check if data is augmented
pub fn is_data_augmented(attributes: &HashSet<DataAttribute>) -> bool {
    attributes.iter().any(|attr| {
        matches!(attr, DataAttribute::Aggregated | DataAttribute::Cleaned)
    })
}

/// Public helper function to check if data is identified (NOT deidentified)
/// This can be called from anywhere in the game to check if data is identified
pub fn is_data_identified(attributes: &HashSet<DataAttribute>) -> bool {
    !attributes.contains(&DataAttribute::DeIdentified)
}

/// Public function to spawn an augmented data indicator sprite with pulse animation
/// Can be called from anywhere in the game to add the augmented visual to an entity
/// 
/// # Arguments
/// * `commands` - Mutable reference to Commands for spawning entities
/// * `position` - World position where the indicator should appear
/// * `parent_entity` - Optional entity to associate with this indicator
/// * `asset_server` - Reference to AssetServer for loading the augmented.png texture
/// 
/// # Returns
/// The Entity ID of the spawned indicator
pub fn spawn_augmented_indicator(
    commands: &mut Commands,
    position: Vec3,
    parent_entity: Option<Entity>,
    asset_server: &AssetServer,
) -> Entity {
    let augmented_texture = asset_server.load("augmented.png");
    
    // Use parent entity index for desync, or random if no parent
    let pulse_offset = if let Some(parent) = parent_entity {
        (parent.index() as f32 * 0.3) % 2.0
    } else {
        (rand::random::<f32>() * 2.0)
    };
    
    commands.spawn((
        Sprite {
            image: augmented_texture,
            custom_size: Some(Vec2::new(12.0, 12.0)), // Small indicator
            ..Default::default()
        },
        Transform::from_translation(position)
            .with_scale(Vec3::splat(1.0)),
        AugmentedIndicator {
            parent_icon: parent_entity.unwrap_or(Entity::PLACEHOLDER),
            base_scale: 1.0,
            time_offset: pulse_offset,
        },
    )).id()
}

/// Public function to add the scanning flash effect to an entity
/// Can be called from anywhere in the game to make an entity flash with the scanning effect
/// 
/// # Arguments
/// * `commands` - Mutable reference to Commands for inserting components
/// * `target_entity` - The entity that should receive the scanning flash effect
/// * `sprite_handle` - Optional sprite handle to add if the entity doesn't have one yet
pub fn add_scanning_flash_effect(
    commands: &mut Commands,
    target_entity: Entity,
    sprite_handle: Option<Handle<Image>>,
) {
    // If a sprite handle is provided, ensure the entity has a sprite component
    if let Some(image) = sprite_handle {
        commands.entity(target_entity).insert(Sprite {
            image,
            ..Default::default()
        });
    }
    
    // Add the scanning flash effect component
    let flash_offset = (target_entity.index() as f32 * 0.25) % 3.0;
    commands.entity(target_entity).insert(ScanningFlashEffect {
        parent_icon: target_entity,
        timer: flash_offset,
        flash_interval: 3.0,
    });
}

/// Public function to spawn both augmented indicator AND add scanning flash to a target entity
/// Convenience function for when you need both effects
/// 
/// # Arguments
/// * `commands` - Mutable reference to Commands
/// * `target_entity` - The entity that should receive the scanning flash
/// * `indicator_position` - World position for the augmented indicator (usually above/beside target)
/// * `asset_server` - Reference to AssetServer
/// 
/// # Returns
/// The Entity ID of the spawned augmented indicator
pub fn spawn_full_data_visualization(
    commands: &mut Commands,
    target_entity: Entity,
    indicator_position: Vec3,
    asset_server: &AssetServer,
) -> Entity {
    // Add scanning flash to the target entity
    add_scanning_flash_effect(commands, target_entity, None);
    
    // Spawn augmented indicator sprite
    spawn_augmented_indicator(commands, indicator_position, Some(target_entity), asset_server)
}

/// Public function to spawn an augmented indicator in UI space (using Node/Style positioning)
/// Perfect for UI elements like tooltips, menus, or HUD displays
/// 
/// # Arguments
/// * `commands` - Mutable reference to Commands
/// * `ui_offset` - UI offset in pixels (e.g., Val::Px(10.0) for right, Val::Px(-10.0) for top)
/// * `parent_entity` - Optional entity to associate with this indicator
/// * `asset_server` - Reference to AssetServer
/// 
/// # Returns
/// The Entity ID of the spawned UI indicator
pub fn spawn_augmented_indicator_ui(
    commands: &mut Commands,
    ui_offset: (Val, Val), // (left/right, top/bottom)
    parent_entity: Option<Entity>,
    asset_server: &AssetServer,
) -> Entity {
    let augmented_texture = asset_server.load("augmented.png");
    
    // Use parent entity index for desync, or random if no parent
    let pulse_offset = if let Some(parent) = parent_entity {
        (parent.index() as f32 * 0.3) % 2.0
    } else {
        (rand::random::<f32>() * 2.0)
    };
    
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: ui_offset.0,
            top: ui_offset.1,
            width: Val::Px(12.0),
            height: Val::Px(12.0),
            ..Default::default()
        },
        ImageNode {
            image: augmented_texture,
            ..Default::default()
        },
        AugmentedIndicator {
            parent_icon: parent_entity.unwrap_or(Entity::PLACEHOLDER),
            base_scale: 1.0,
            time_offset: pulse_offset,
        },
    )).id()
}

/// Public function to spawn a data type icon with optional augmentations and identification status
/// This creates a complete data visualization ready to use in world space or UI
/// 
/// # Arguments
/// * `commands` - Mutable reference to Commands
/// * `data_type` - The type of data to visualize (Biometric, Economic, etc.)
/// * `attributes` - Set of data attributes (Aggregated, Cleaned, DeIdentified, etc.)
/// * `position` - World position or UI position for the icon
/// * `is_ui` - Whether this is a UI element (uses Node) or world element (uses Transform)
/// * `game_assets` - Reference to GameAssets for texture atlas
/// * `asset_server` - Reference to AssetServer for loading textures
/// 
/// # Returns
/// Tuple of (icon_entity, optional_augmented_indicator_entity)
pub fn spawn_data_type_with_augmentations(
    commands: &mut Commands,
    data_type: BasicDataType,
    attributes: HashSet<DataAttribute>,
    position: Vec3, // For world space, or convert to UI if is_ui=true
    is_ui: bool,
    game_assets: &GameAssets,
    asset_server: &AssetServer,
) -> (Entity, Option<Entity>) {
    let has_augmentation = is_data_augmented(&attributes);
    let is_identified = is_data_identified(&attributes);
    
    // Get the sprite index for this data type
    let sprite_index = *game_assets.data_type_icons_large.get(&data_type).unwrap_or(&0);
    let (texture, layout) = game_assets.get_atlas(AtlasId::LargeSprites);
    
    let icon_entity = if is_ui {
        // Spawn as UI element - NOTE: UI nodes with atlases need special handling
        let entity_id = commands.spawn((
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                position_type: PositionType::Absolute,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                ..Default::default()
            },
            ImageNode {
                image: texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: layout.clone(),
                    index: sprite_index,
                }),
                ..Default::default()
            },
        )).id();
        
        // Add data type icon component and scanning flash separately
        commands.entity(entity_id).insert(DataTypeIcon {
            data_type,
            parent_source: Entity::PLACEHOLDER,
        });
        
        // Add scanning flash for identified data
        if is_identified {
            let flash_offset = (rand::random::<f32>() * 3.0) % 3.0;
            commands.entity(entity_id).insert(ScanningFlashEffect {
                parent_icon: Entity::PLACEHOLDER, // Will be set after spawn
                timer: flash_offset,
                flash_interval: 3.0,
            });
        }
        
        entity_id
    } else {
        // Spawn as world space sprite
        let entity_id = commands.spawn((
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::splat(32.0)),
                texture_atlas: Some(TextureAtlas {
                    layout: layout.clone(),
                    index: sprite_index,
                }),
                ..Default::default()
            },
            Transform::from_translation(position),
        )).id();
        
        // Add data type icon component and scanning flash separately
        commands.entity(entity_id).insert(DataTypeIcon {
            data_type,
            parent_source: Entity::PLACEHOLDER,
        });
        
        // Add scanning flash for identified data
        if is_identified {
            let flash_offset = (rand::random::<f32>() * 3.0) % 3.0;
            commands.entity(entity_id).insert(ScanningFlashEffect {
                parent_icon: Entity::PLACEHOLDER, // Will be set after spawn
                timer: flash_offset,
                flash_interval: 3.0,
            });
        }
        
        entity_id
    };
    
    // Spawn augmented indicator if needed
    let augmented_entity = if has_augmentation {
        if is_ui {
            // UI space indicator - positioned relative to icon
            Some(spawn_augmented_indicator_ui(
                commands,
                (Val::Px(20.0), Val::Px(-2.0)), // Top-right corner
                Some(icon_entity),
                asset_server,
            ))
        } else {
            // World space indicator - positioned above and to the right
            let indicator_pos = position + Vec3::new(10.0, 14.0, 0.2);
            Some(spawn_augmented_indicator(
                commands,
                indicator_pos,
                Some(icon_entity),
                asset_server,
            ))
        }
    } else {
        None
    };
    
    (icon_entity, augmented_entity)
}

pub struct SourceVisualsPlugin;

impl Plugin for SourceVisualsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                spawn_source_backgrounds,
                update_source_data_icons,
                animate_scanning_flash,
                animate_floating_icons,
                animate_augmented_pulse,
            ));
    }
}
