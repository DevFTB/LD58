use bevy::{
    color::palettes::css::{ANTIQUE_WHITE, BROWN}, math::I8Vec2, prelude::*
};

pub struct UIPlugin;

const BUILDING_BAR_WIDTH_PCT: f32 = 70.0;
const BUILDING_BAR_HEIGHT_PCT: f32 = 12.0;
const BUILDING_TILE_SIZE: i64 = 64;

const RIGHT_BAR_WIDTH_PCT: f32 = 20.0;

// #[derive(Event)]
// pub struct ConstructBuildingEvent {
//     pub building: UIBuilding,
//     pub grid_position: I8Vec2,
// }

#[derive(Component, Clone)]
pub struct UIBuilding{
    sprite_path: String,
}

#[derive(Component)]
pub struct SelectedBuilding;

#[derive(Resource)]
pub struct SelectedBuildingType(pub Option<UIBuilding>);

#[derive(Resource)]
pub struct JustSelected(pub bool);

impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(SelectedBuildingType(None))
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
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
        UIBuilding{sprite_path: String::from(r"buildings/building_placeholder.png")},
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
            let mut image_node = ImageNode::new(asset_server.load(&building.sprite_path));
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
) {
    for (_entity, interaction, building) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Remove any existing selected building
                for selected_entity in selected_query.iter() {
                    commands.entity(selected_entity).despawn();
                }
                // Set the selected building type
                selected_building_type.0 = Some(building.clone());
                just_selected.0 = true;
                
                // Get initial mouse position
                let initial_position = get_mouse_world_position(&windows, &camera_query)
                    .unwrap_or(Vec3::ZERO);
                
                // Spawn a dragged building sprite at mouse position
                commands.spawn((
                    SelectedBuilding,
                    Sprite {
                        image: asset_server.load(&building.sprite_path),
                        custom_size: Some(Vec2::new(64.0, 64.0)),
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
    mut selected_query: Query<&mut Transform, With<SelectedBuilding>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<crate::grid::Grid>,
) {
    if let Some(world_position) = get_mouse_world_position(&windows, &camera_query) {
        // Snap to grid
        let grid_x = ((world_position.x / grid.scale).round()) as i8;
        let grid_y = ((world_position.y / grid.scale).round()) as i8;
        let snapped_position = Vec3::new(
            grid_x as f32 * grid.scale,
            grid_y as f32 * grid.scale,
            100.0,
        );
        
        for mut transform in selected_query.iter_mut() {
            transform.translation = snapped_position;
        }
    }
}

fn handle_building_rotate(
    key_input: Res<ButtonInput<KeyCode>>,
    mut selected_query: Query<(Entity, &mut Transform), With<SelectedBuilding>>,
) {
    if key_input.just_pressed(KeyCode::KeyR) {
        for (_entity, mut building_transform) in &mut selected_query {
            building_transform.rotate_z(std::f32::consts::FRAC_PI_2);
        }
    }
}


fn handle_placement_click(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    selected_query: Query<Entity, With<SelectedBuilding>>,
    mut selected_building_type: ResMut<SelectedBuildingType>,
    just_selected: Res<JustSelected>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    grid: Res<crate::grid::Grid>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) && !just_selected.0 {
        // Check if we have a selected building
        if selected_building_type.0.is_some() {
            // Get mouse position
            if let Some(world_position) = get_mouse_world_position(&windows, &camera_query) {
                // Calculate grid position
                let grid_x = ((world_position.x / grid.scale).round()) as i8;
                let grid_y = ((world_position.y / grid.scale).round()) as i8;
                let _grid_position = I8Vec2::new(grid_x, grid_y);
                
                // TODO: Send ConstructBuildingEvent here
                // construct_events.send(ConstructBuildingEvent {
                //     building: building.clone(),
                //     grid_position,
                // });
                
                // Despawn the dragged building
                for entity in selected_query.iter() {
                    commands.entity(entity).despawn();
                }
                
                // Clear selection
                selected_building_type.0 = None;
            }
        }
    }
}

fn reset_just_selected(mut just_selected: ResMut<JustSelected>) {
    just_selected.0 = false;
}

