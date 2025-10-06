use crate::grid::{GridAtlasSprite, GridPosition, Orientation};
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

        match data.sprite {
            SpriteResource::Atlas(index) => {
                commands.entity(id).insert(GridAtlasSprite {
                    atlas_index: index,
                    grid_width: data.grid_width,
                    grid_height: data.grid_height,
                    orientation,
                });
            }
            SpriteResource::Sprite(image) => {
                commands.entity(id).insert(Sprite { image, ..default() });
            }
        }

        id
    }

    fn data(&self) -> BuildingData;
}

#[derive(Clone)]
pub enum SpriteResource {
    Atlas(usize), // Sprite index in the texture atlas (works for all building sizes)
    Sprite(Handle<Image>), // Fallback to individual sprite file
}

pub struct SpriteComponent {
    pub grid_width: i64,
    pub grid_height: i64,
    pub sprite: SpriteResource,
}

#[derive(Clone)]
pub struct BuildingData {
    // Common UI fields
    pub sprite: SpriteResource,
    pub grid_width: i64,
    pub grid_height: i64,
    pub cost: i32,
    pub name: String,
}

impl BuildingData {
    pub fn get_sprite_component(&self) -> SpriteComponent {
        SpriteComponent {
            sprite: self.sprite.clone(),
            grid_height: self.grid_height,
            grid_width: self.grid_width,
        }
    }
}
