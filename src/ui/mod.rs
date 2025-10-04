use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, BROWN}, math::I8Vec2, prelude::*
};

use crate::things::buildings::BuildingType;
use crate::grid::{WorldMap, GridPosition, calculate_occupied_cells, are_positions_free};

pub struct UIPlugin;

const BUILDING_BAR_WIDTH_PCT: f32 = 70.0;
const BUILDING_BAR_HEIGHT_PCT: f32 = 12.0;
const BUILDING_TILE_SIZE: i64 = 64;

const RIGHT_BAR_WIDTH_PCT: f32 = 20.0;

#[derive(Event, Message)]
pub struct ConstructBuildingEvent {
    pub building_type: BuildingType,
    pub grid_position: I8Vec2,
    pub rotation: u8,
}

#[derive(Component, Clone)]
pub struct UIBuilding {
    pub building_type: BuildingType,
}

#[derive(Component)]
pub struct SelectedBuilding;

#[derive(Component)]
pub struct BuildingRotation(pub u8); // 0, 1, 2, 3 for 0°, 90°, 180°, 270°

#[derive(Resource)]
pub struct SelectedBuildingType(pub Option<BuildingType>);

#[derive(Resource)]
pub struct JustSelected(pub bool);

impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<ConstructBuildingEvent>()
            .insert_resource(SelectedBuildingType(None))
            .insert_resource(JustSelected(false))
            .add_systems(Startup, startup)
            .add_systems(Update, handle_building_click)
            .add_systems(Update, update_selected_building_position)
            .add_systems(Update, handle_placement_click)
            .add_systems(Update, handle_building_rotate)
            .add_systems(Update, reset_just_selected);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // i tried really hard to abstract this into a .ron file for way too long but failed horribly. hence what is currently here
    let buildings = [
        UIBuilding{building_type: BuildingType::Splitter2x1},
        UIBuilding{building_type: BuildingType::Splitter3x1},
        UIBuilding{building_type: BuildingType::Splitter4x1},
        UIBuilding{building_type: BuildingType::Splitter2x1},
        UIBuilding{building_type: BuildingType::Splitter3x1},
        UIBuilding{building_type: BuildingType::Splitter4x1},
        UIBuilding{building_type: BuildingType::Splitter2x1},
        UIBuilding{building_type: BuildingType::Splitter3x1},
        UIBuilding{building_type: BuildingType::Splitter4x1},
        UIBuilding{building_type: BuildingType::Decoupler},
    ];

    

    // spawn the bottom bar with factory draggables
    commands.spawn((
        Node {
            width: percent(BUILDING_BAR_WIDTH_PCT),
            height: percent(BUILDING_BAR_HEIGHT_PCT),
            display: Display::Flex,
            position_type: PositionType::Absolute,
            top: percent(100.0 - BUILDING_BAR_HEIGHT_PCT),
            left: percent((100.0 - BUILDING_BAR_WIDTH_PCT)/2.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(ANTIQUE_WHITE.into()),
        ZIndex(1), // Ensure UI renders above sprites
    )).with_children(|parent|{
        for building in &buildings {
            let mut image_node = ImageNode::new(asset_server.load(&building.building_type.data().sprite_path));
            image_node.image_mode = NodeImageMode::Stretch;

            parent.spawn((
                Node {
                    width: px(BUILDING_TILE_SIZE),
                    height: px(BUILDING_TILE_SIZE),
                    ..default()
                },
                image_node,
                // BackgroundColor(GRAY.into()),
                building.clone(),
                Interaction::None,
                Button,
                Transform::from_xyz(0.0, 0.0, 100.0),

            ));
        }
    });

    // spawn the right bar with other information: contracts + newsfeed atm
    commands.spawn((
        Node {
            width: percent(RIGHT_BAR_WIDTH_PCT),
            height: percent(100),
            display: Display::Flex,
            position_type: PositionType::Absolute,
            top: percent(0),
            left: percent(100.0 - RIGHT_BAR_WIDTH_PCT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            // align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BROWN.into()),
        ZIndex(-1),
    ));
}

fn get_mouse_world_position(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec3> {
    if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), camera_query.single()) {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok(world_position) = camera.viewport_to_world(camera_transform, cursor_position) {
                return Some(world_position.origin);
            }
        }
    }
    None
}

fn handle_building_click(
    mut commands: Commands,
    mut interaction_query: Query<
        (Entity, &Interaction, &UIBuilding),
        (Changed<Interaction>, With<UIBuilding>),
    >,
    asset_server: Res<AssetServer>,
    mut selected_building_type: ResMut<SelectedBuildingType>,
    mut just_selected: ResMut<JustSelected>,
    selected_query: Query<Entity, With<SelectedBuilding>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<crate::grid::Grid>,
) {
    for (_entity, interaction, building) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Remove any existing selected building
                for selected_entity in selected_query.iter() {
                    commands.entity(selected_entity).despawn();
                }
                // Set the selected building type
                selected_building_type.0 = Some(building.building_type);
                just_selected.0 = true;
                
                // Get initial mouse position
                let initial_position = get_mouse_world_position(&windows, &camera_query)
                    .unwrap_or(Vec3::ZERO);
                
                // Spawn a dragged building sprite at mouse position
                let data = building.building_type.data();
                let sprite_size = Vec2::new(
                    data.grid_width as f32 * grid.scale,
                    data.grid_height as f32 * grid.scale,
                );
                
                commands.spawn((
                    SelectedBuilding,
                    BuildingRotation(0),
                    Sprite {
                        image: asset_server.load(&data.sprite_path),
                        custom_size: Some(sprite_size),
                        ..default()
                    },
                    Transform::from_xyz(initial_position.x, initial_position.y, 100.0),
                    ZIndex(10), // Ensure it renders above UI
                ));
            }
            _ => {}
        }
    }
}

fn update_selected_building_position(
    mut selected_query: Query<(&mut Transform, &mut Sprite, &BuildingRotation), With<SelectedBuilding>>,
    selected_building_type: Res<SelectedBuildingType>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<crate::grid::Grid>,
    world_map: Res<WorldMap>,
) {
    if let Some(world_position) = get_mouse_world_position(&windows, &camera_query) {
        if let Some(building_type) = &selected_building_type.0 {
            let data = building_type.data();
            
            for (mut transform, mut sprite, rotation) in selected_query.iter_mut() {
                // Apply rotation to dimensions for snapping
                let (width, height) = if rotation.0 % 2 == 1 {
                    (data.grid_height, data.grid_width) // Swap for odd rotations
                } else {
                    (data.grid_width, data.grid_height)
                };
                
                // Snap to grid - center building on mouse cursor
                let mouse_grid_x = world_position.x / grid.scale;
                let mouse_grid_y = world_position.y / grid.scale;
                
                // Calculate anchor position by snapping center to nearest valid position
                let center_slot_x = mouse_grid_x.round();
                let center_slot_y = mouse_grid_y.round();
                
                let anchor_x = (center_slot_x - (width - 1) as f32 / 2.0) as i8;
                let anchor_y = (center_slot_y - (height - 1) as f32 / 2.0) as i8;
                let grid_pos = I8Vec2::new(anchor_x, anchor_y);
                
                // Calculate sprite center position relative to anchor
                let center_x = (anchor_x as f32 + (width - 1) as f32 / 2.0) * grid.scale;
                let center_y = (anchor_y as f32 + (height - 1) as f32 / 2.0) * grid.scale;
                
                let snapped_position = Vec3::new(center_x, center_y, 100.0);
                
                transform.translation = snapped_position;

                // Check if positions are occupied
                let occupied_positions = calculate_occupied_cells(grid_pos, width, height)
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
}

fn handle_building_rotate(
    key_input: Res<ButtonInput<KeyCode>>,
    mut selected_query: Query<(Entity, &mut Transform, &mut BuildingRotation), With<SelectedBuilding>>,
    selected_building_type: Res<SelectedBuildingType>,
) {
    if key_input.just_pressed(KeyCode::KeyR) {
        if let Some(_building_type) = &selected_building_type.0 {
            for (_entity, mut building_transform, mut rotation) in &mut selected_query {
                rotation.0 = (rotation.0 + 1) % 4;
                building_transform.rotate_z(-std::f32::consts::FRAC_PI_2);
            }
        }
    }
}


fn handle_placement_click(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    selected_query: Query<(Entity, &BuildingRotation), With<SelectedBuilding>>,
    mut selected_building_type: ResMut<SelectedBuildingType>,
    just_selected: Res<JustSelected>,
    mut construct_events: MessageWriter<ConstructBuildingEvent>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<crate::grid::Grid>,
    world_map: Res<WorldMap>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) && !just_selected.0 {
        // Check if we have a selected building
        if let Some(building_type) = &selected_building_type.0 {
            // Get mouse position
            if let Some(world_position) = get_mouse_world_position(&windows, &camera_query) {
                // Get building data and apply rotation first
                let data = building_type.data();
                let rotation = selected_query.iter().next().map(|(_, r)| r.0).unwrap_or(0);
                
                let (width, height) = if rotation % 2 == 1 {
                    (data.grid_height, data.grid_width) // Swap for odd rotations
                } else {
                    (data.grid_width, data.grid_height)
                };
                
                // Calculate grid position by centering on mouse
                let mouse_grid_x = world_position.x / grid.scale;
                let mouse_grid_y = world_position.y / grid.scale;
                let center_slot_x = mouse_grid_x.round();
                let center_slot_y = mouse_grid_y.round();
                
                let anchor_x = (center_slot_x - (width - 1) as f32 / 2.0) as i8;
                let anchor_y = (center_slot_y - (height - 1) as f32 / 2.0) as i8;
                let base_position = I8Vec2::new(anchor_x, anchor_y);

                let occupied_positions = calculate_occupied_cells(base_position, width, height)
                    .into_iter()
                    .map(GridPosition)
                    .collect::<Vec<_>>();
                
                // Only place if positions are free
                if are_positions_free(&world_map, &occupied_positions) {
                    // Send construction event
                    construct_events.write(ConstructBuildingEvent {
                        building_type: *building_type,
                        grid_position: base_position,
                        rotation,
                    });
                    
                    // Despawn the dragged building
                    for (entity, _rotation) in selected_query.iter() {
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

fn reset_just_selected(mut just_selected: ResMut<JustSelected>) {
    just_selected.0 = false;
}

