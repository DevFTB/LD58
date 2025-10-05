use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, BROWN},
    math::I8Vec2,
    prelude::*,
};

use crate::grid::{
    GridPosition, WorldMap, are_positions_free, calculate_occupied_cells,
    calculate_occupied_cells_rotated,
};
use crate::things::buildings::BuildingType;

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
            .add_systems(
                Update,
                (
                    handle_building_click,
                    handle_building_rotate,
                    update_selected_building_position,
                    handle_placement_click,
                    reset_just_selected,
                )
                    .chain(),
            );
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // i tried really hard to abstract this into a .ron file for way too long but failed horribly. hence what is currently here
    let buildings = [
        UIBuilding {
            building_type: BuildingType::Splitter2x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter3x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter4x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter2x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter3x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter4x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter2x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter3x1,
        },
        UIBuilding {
            building_type: BuildingType::Splitter4x1,
        },
        UIBuilding {
            building_type: BuildingType::Decoupler,
        },
    ];

    // spawn the bottom bar with factory draggables
    commands
        .spawn((
            Node {
                width: percent(BUILDING_BAR_WIDTH_PCT),
                height: percent(BUILDING_BAR_HEIGHT_PCT),
                display: Display::Flex,
                position_type: PositionType::Absolute,
                top: percent(100.0 - BUILDING_BAR_HEIGHT_PCT),
                left: percent((100.0 - BUILDING_BAR_WIDTH_PCT) / 2.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(ANTIQUE_WHITE.into()),
            ZIndex(1), // Ensure UI renders above sprites
        ))
        .with_children(|parent| {
            for building in &buildings {
                let mut image_node =
                    ImageNode::new(asset_server.load(&building.building_type.data().sprite_path));
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
                let initial_position =
                    get_mouse_world_position(&windows, &camera_query).unwrap_or(Vec3::ZERO);

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
    mut selected_query: Query<
        (&mut Transform, &mut Sprite, &BuildingRotation),
        With<SelectedBuilding>,
    >,
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
                // Convert mouse position to grid coordinates - this is our anchor cell
                let snapped_grid_pos = grid.world_to_grid(world_position.xy());

                // Get the world center of the anchor cell
                let anchor_cell_center = grid.grid_to_world_center(&snapped_grid_pos);

                // Calculate offset from anchor to sprite center based on rotation
                // For a 3x1 building, the center is (width-1)/2 cells away from anchor
                let offset = ((data.grid_width - 1) as f32 / 2.0) * grid.scale;

                let (sprite_center_x, sprite_center_y) = match rotation.0 % 4 {
                    0 => (anchor_cell_center.x + offset, anchor_cell_center.y), // extends right
                    1 => (anchor_cell_center.x, anchor_cell_center.y - offset), // extends down
                    2 => (anchor_cell_center.x - offset, anchor_cell_center.y), // extends left
                    3 => (anchor_cell_center.x, anchor_cell_center.y + offset), // extends up
                    _ => unreachable!(),
                };

                let snapped_position = Vec3::new(sprite_center_x, sprite_center_y, 100.0);
                transform.translation = snapped_position;

                // Check if positions are occupied using rotated cell calculation
                let occupied_positions = calculate_occupied_cells_rotated(
                    *snapped_grid_pos,
                    data.grid_width,
                    data.grid_height,
                    rotation.0,
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
}

fn handle_building_rotate(
    key_input: Res<ButtonInput<KeyCode>>,
    mut selected_query: Query<
        (Entity, &mut Transform, &mut BuildingRotation),
        With<SelectedBuilding>,
    >,
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
                // Get building data and rotation
                let data = building_type.data();
                let rotation = selected_query.iter().next().map(|(_, r)| r.0).unwrap_or(0);

                // Convert mouse position to grid coordinates - this is the anchor cell
                let snapped_grid_pos = grid.world_to_grid(world_position.xy());

                // The snapped grid position IS the anchor
                let base_position = *snapped_grid_pos;

                // Calculate occupied positions using rotation-aware function
                let occupied_positions = calculate_occupied_cells_rotated(
                    base_position,
                    data.grid_width,
                    data.grid_height,
                    rotation,
                )
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
