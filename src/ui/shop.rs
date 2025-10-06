use bevy::{color::palettes::css::ANTIQUE_WHITE, prelude::*};
use std::sync::Arc;

use crate::assets::GameAssets;
use crate::factory::buildings::aggregator::Aggregator;
use crate::factory::buildings::buildings::{Building, SpriteResource};
use crate::factory::buildings::combiner::Combiner;
use crate::factory::buildings::splitter::Splitter;
use crate::factory::physical::PhysicalLink;
use crate::factory::ConstructBuildingEvent;
use crate::grid::{
    are_positions_free, calculate_occupied_cells_rotated, Grid, GridPosition, Orientation, WorldMap,
};
use crate::ui::BlocksWorldClicks;

pub const BUILDING_BAR_WIDTH_PCT: f32 = 70.0;
pub const BUILDING_BAR_HEIGHT_PCT: f32 = 12.0;
const BUILDING_TILE_SIZE: i64 = 64;

#[derive(Component, Clone)]
pub struct UIBuilding {
    pub building_type: Arc<dyn Building>,
}

#[derive(Component)]
pub struct SelectedBuilding;

#[derive(Component)]
pub struct BuildingOrientation(pub Orientation);

#[derive(Resource)]
pub struct SelectedBuildingType(pub Option<Arc<dyn Building>>);

/// Spawns the building shop UI bar at the bottom of the screen
pub fn spawn_building_shop(mut commands: Commands, assets: Res<GameAssets>) {
    let buildings = [
        UIBuilding {
            building_type: Arc::new(Splitter {
                source_count: 2,
                throughput: 5.0,
            }),
        },
        UIBuilding {
            building_type: Arc::new(Splitter {
                source_count: 3,
                throughput: 5.0,
            }),
        },
        UIBuilding {
            building_type: Arc::new(Splitter {
                source_count: 4,
                throughput: 5.0,
            }),
        },
        UIBuilding {
            building_type: Arc::new(Combiner {
                sink_count: 2,
                throughput: 5.0,
            }),
        },
        UIBuilding {
            building_type: Arc::new(Combiner {
                sink_count: 3,
                throughput: 5.0,
            }),
        },
        UIBuilding {
            building_type: Arc::new(Combiner {
                sink_count: 4,
                throughput: 5.0,
            }),
        },
        UIBuilding {
            building_type: Arc::new(Aggregator { throughput: 5.0 }),
        },
        UIBuilding {
            building_type: Arc::new(PhysicalLink { throughput: 50.0 }),
        },
    ];

    // spawn the bottom bar with factory draggables
    commands
        .spawn((
            Node {
                width: Val::Percent(BUILDING_BAR_WIDTH_PCT),
                height: Val::Percent(BUILDING_BAR_HEIGHT_PCT),
                display: Display::Flex,
                position_type: PositionType::Absolute,
                top: Val::Percent(100.0 - BUILDING_BAR_HEIGHT_PCT),
                left: Val::Percent((100.0 - BUILDING_BAR_WIDTH_PCT) / 2.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ANTIQUE_WHITE.into()),
            ZIndex(1), // Ensure UI renders above sprites
            BlocksWorldClicks,
        ))
        .with_children(|parent| {
            for building in &buildings {
                let data = building.building_type.data();
                let mut image_node = match &data.sprite {
                    Some(SpriteResource::Atlas(atlas_id, index)) => {
                        let (texture, layout) = assets.get_atlas(*atlas_id);
                        ImageNode::from_atlas_image(
                            texture,
                            TextureAtlas {
                                layout,
                                index: *index,
                            },
                        )
                    },
                    Some(SpriteResource::Machine(machine_type, variant)) => {
                        if let Some((atlas_id, index)) = assets.machine_sprite(*machine_type, *variant) {
                            let (texture, layout) = assets.get_atlas(atlas_id);
                            ImageNode::from_atlas_image(
                                texture,
                                TextureAtlas {
                                    layout,
                                    index,
                                },
                            )
                        } else {
                            ImageNode::default()
                        }
                    },
                    Some(SpriteResource::Sprite(path)) => ImageNode::new(path.clone()),
                    None => ImageNode::default(),
                };
                image_node.image_mode = NodeImageMode::Stretch;

                parent.spawn((
                    Node {
                        width: Val::Px(BUILDING_TILE_SIZE as f32),
                        height: Val::Px(BUILDING_TILE_SIZE as f32),
                        ..default()
                    },
                    image_node,
                    building.clone(),
                    Interaction::None,
                    Button,
                    Transform::from_xyz(0.0, 0.0, 100.0),
                ));
            }
        });
}

fn get_mouse_world_position(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec3> {
    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), camera_query.single())
    {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok(world_position) = camera.viewport_to_world(camera_transform, cursor_position)
            {
                return Some(world_position.origin);
            }
        }
    }
    None
}

pub fn handle_building_click(
    mut commands: Commands,
    mut interaction_query: Query<
        (Entity, &Interaction, &UIBuilding),
        (Changed<Interaction>, With<UIBuilding>),
    >,
    selected_query: Query<Entity, With<SelectedBuilding>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<Grid>,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    mut selected_building_type: ResMut<SelectedBuildingType>,
) {
    for (_entity, interaction, building) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Remove any existing selected building
            for selected_entity in selected_query.iter() {
                commands.entity(selected_entity).despawn();
            }
            // Set the selected building type
            selected_building_type.0 = Some(building.building_type.clone());

            // Get initial mouse position
            let initial_position =
                get_mouse_world_position(&windows, &camera_query).unwrap_or(Vec3::ZERO);

            // Spawn a dragged building sprite at mouse position
            let data = building.building_type.data();
            let sprite_size = Vec2::new(
                data.grid_width as f32 * grid.scale,
                data.grid_height as f32 * grid.scale,
            );

            // Create sprite based on SpriteResource type
            let sprite = match &data.sprite {
                Some(SpriteResource::Atlas(atlas_id, index)) => {
                    let (texture, layout) = assets.get_atlas(*atlas_id);
                    Sprite {
                        image: texture,
                        custom_size: Some(sprite_size),
                        texture_atlas: Some(TextureAtlas {
                            layout,
                            index: *index,
                        }),
                        ..default()
                    }
                },
                Some(SpriteResource::Machine(machine_type, variant)) => {
                    if let Some((atlas_id, index)) = assets.machine_sprite(*machine_type, *variant) {
                        let (texture, layout) = assets.get_atlas(atlas_id);
                        Sprite {
                            image: texture,
                            custom_size: Some(sprite_size),
                            texture_atlas: Some(TextureAtlas {
                                layout,
                                index,
                            }),
                            ..default()
                        }
                    } else {
                        Sprite::default()
                    }
                },
                Some(SpriteResource::Sprite(image)) => Sprite {
                    image: image.clone(),
                    custom_size: Some(sprite_size),
                    ..default()
                },
                None => Sprite::default(),
            };

            commands.spawn((
                SelectedBuilding,
                BuildingOrientation(Orientation::default()),
                sprite,
                Transform::from_xyz(initial_position.x, initial_position.y, 100.0),
                ZIndex(10), // Ensure it renders above UI
            ));
        }
    }
}

pub fn update_selected_building_position(
    mut selected_query: Query<
        (&mut Transform, &mut Sprite, &BuildingOrientation),
        With<SelectedBuilding>,
    >,
    selected_building_type: Res<SelectedBuildingType>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<crate::grid::Grid>,
    world_map: Res<WorldMap>,
) {
    if let Some(world_position) = get_mouse_world_position(&windows, &camera_query)
        && let Some(building_type) = &selected_building_type.0
    {
        let data = building_type.data();

        for (mut transform, mut sprite, orientation) in selected_query.iter_mut() {
            // Snap mouse position to grid to get the anchor cell
            let snapped_grid_pos = grid.world_to_grid(world_position.xy());

            // Use the shared utility function to calculate sprite position
            let sprite_pos = grid.calculate_building_sprite_position(
                &snapped_grid_pos,
                data.grid_width,
                data.grid_height,
                orientation.0,
            );

            let snapped_position = Vec3::new(sprite_pos.x, sprite_pos.y, 100.0);

            transform.translation = snapped_position;

            // Check if positions are occupied
            let occupied_positions = calculate_occupied_cells_rotated(
                *snapped_grid_pos,
                data.grid_width,
                data.grid_height,
                orientation.0,
            )
            .into_iter()
            .map(GridPosition)
            .collect::<Vec<_>>();

            if are_positions_free(&world_map, &occupied_positions) {
                // Valid placement - normal color
                sprite.color = Color::WHITE;
            } else {
                // Invalid placement - tint red
                sprite.color = Color::srgb(1.0, 0.5, 0.5);
            }
        }
    }
}

pub fn handle_building_rotate(
    key_input: Res<ButtonInput<KeyCode>>,
    mut selected_query: Query<
        (Entity, &mut Transform, &mut BuildingOrientation),
        With<SelectedBuilding>,
    >,
    selected_building_type: Res<SelectedBuildingType>,
) {
    if key_input.just_pressed(KeyCode::KeyR)
        && let Some(_building_type) = &selected_building_type.0
    {
        for (_entity, mut building_transform, mut orientation) in &mut selected_query {
            orientation.0 = orientation.0.rotate_clockwise();
            building_transform.rotate_z(-std::f32::consts::FRAC_PI_2);
        }
    }
}

pub fn handle_building_flip(
    key_input: Res<ButtonInput<KeyCode>>,
    mut selected_query: Query<
        (Entity, &mut Sprite, &mut BuildingOrientation),
        With<SelectedBuilding>,
    >,
    selected_building_type: Res<SelectedBuildingType>,
) {
    if key_input.just_pressed(KeyCode::KeyF)
        && let Some(_building_type) = &selected_building_type.0
    {
        for (_entity, mut sprite, mut orientation) in &mut selected_query {
            // Toggle flip state
            orientation.0 = orientation.0.toggle_flip();

            // Always apply sprite flip_x when flipped
            sprite.flip_x = orientation.0.flipped;
        }
    }
}

pub fn handle_placement_click(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    selected_query: Query<(Entity, &BuildingOrientation), With<SelectedBuilding>>,
    mut selected_building_type: ResMut<SelectedBuildingType>,
    mut construct_events: MessageWriter<ConstructBuildingEvent>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<crate::grid::Grid>,
    world_map: Res<WorldMap>,
    ui_blocker_query: Query<&Interaction, With<BlocksWorldClicks>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Check if cursor is over any BlocksWorldClicks UI panel
        for interaction in ui_blocker_query.iter() {
            if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
                // Cursor is over a UI panel, don't place building
                return;
            }
        }

        // Check if we have a selected building
        if let Some(building_type) = &selected_building_type.0 {
            // Get mouse position
            if let Some(world_position) = get_mouse_world_position(&windows, &camera_query) {
                // Get building data and orientation
                let data = building_type.data();
                let orientation = selected_query
                    .iter()
                    .next()
                    .map(|(_, o)| o.0)
                    .unwrap_or_default();

                // Convert mouse position to grid coordinates - this is the anchor cell
                let snapped_grid_pos = grid.world_to_grid(world_position.xy());

                // The snapped grid position IS the anchor
                let base_position = *snapped_grid_pos;

                // Calculate occupied positions
                let occupied_positions = calculate_occupied_cells_rotated(
                    base_position,
                    data.grid_width,
                    data.grid_height,
                    orientation,
                )
                .into_iter()
                .map(GridPosition)
                .collect::<Vec<_>>();

                // Only place if positions are free
                if are_positions_free(&world_map, &occupied_positions) {
                    // Send construction event with:
                    // - rotation: The actual rotation direction (not flipped)
                    // - flipped: The flip state - building spawn system will handle the flip
                    construct_events.write(ConstructBuildingEvent {
                        building: building_type.clone(),
                        grid_position: base_position,
                        orientation,
                    });

                    // Despawn the dragged building
                    for (entity, _) in selected_query.iter() {
                        commands.entity(entity).despawn();
                    }

                    // Clear selection
                    selected_building_type.0 = None;
                }
                // If occupied, do nothing - building stays selected and tinted red
            }
        }
    }
}
