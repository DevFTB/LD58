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
            Some(SpriteResource::Atlas(index)) => {
                commands.entity(id).insert(GridAtlasSprite {
                    atlas_index: index,
                    grid_width: data.grid_width,
                    grid_height: data.grid_height,
                    orientation,
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
    Atlas(usize), // Sprite index in the texture atlas (works for all building sizes)
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
