use crate::grid::{GridAtlasSprite, GridPosition, Orientation};
use crate::ui::tooltip::attach_tooltip;
use bevy::prelude::*;

pub trait Building: Send + Sync {
    fn spawn_naked(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity;

    fn spawn(
        &self,
        commands: &mut Commands,
        position: GridPosition,
        orientation: Orientation,
    ) -> Entity {
        let id = self.spawn_naked(commands, position, orientation);
        let data = self.data();

        attach_tooltip(commands, id);

        match data.sprite {
            Some(SpriteResource::Atlas(atlas_id, index)) => {
                commands.entity(id).insert(GridAtlasSprite {
                    atlas_id,
                    atlas_index: index,
                    grid_width: data.grid_width,
                    grid_height: data.grid_height,
                    orientation,
                });
            }
            Some(SpriteResource::Machine(machine_type, variant)) => {
                // Convert Machine to Atlas - atlas_id is derived from variant
                // Index is determined by machine type (handled in GameAssets)
                // We need to access GameAssets, which requires a deferred command
                let grid_width = data.grid_width;
                let grid_height = data.grid_height;
                commands.queue(move |world: &mut World| {
                    if let Some(game_assets) = world.get_resource::<crate::assets::GameAssets>() {
                        if let Some((atlas_id, index)) = game_assets.machine_sprite(machine_type, variant) {
                            if let Ok(mut entity) = world.get_entity_mut(id) {
                                entity.insert(GridAtlasSprite {
                                    atlas_id,
                                    atlas_index: index,
                                    grid_width,
                                    grid_height,
                                    orientation,
                                });
                            }
                        }
                    }
                });
            }
            Some(SpriteResource::Sprite(image)) => {
                commands.entity(id).insert(Sprite { image, ..default() });
            }
            None => {}
        };

        id
    }

    fn data(&self) -> BuildingData;
}

#[derive(Clone)]
pub enum SpriteResource {
    Atlas(crate::assets::AtlasId, usize), // (AtlasId, sprite_index) - which atlas and index within it
    Machine(crate::assets::MachineType, crate::assets::MachineVariant), // Machine type and variant - atlas/index derived automatically
    Sprite(Handle<Image>), // Fallback to individual sprite file
}

#[derive(Clone)]
pub struct BuildingData {
    // Common UI fields
    pub sprite: Option<SpriteResource>,
    pub grid_width: i64,
    pub grid_height: i64,
    pub cost: i32,
    pub name: String,
}
