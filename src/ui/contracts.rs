use bevy::prelude::*;
use crate::{
    contracts::{AssociatedWithSink, Contract, ContractDescription, ContractFulfillment, ContractFulfillmentStatus, ContractStatus},
    grid::GridPosition,
    grid::Grid,
    ui::BlocksWorldScroll,
    factory::buildings::sink::SinkBuilding,
    ui::interactive_event::ScalableText,
    assets::GameAssets,
    factory::logical::Dataset,
};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
};
use crate::camera::focus_camera_on_grid_pos;

#[derive(Component)]
pub struct ContractAcceptButton;

#[derive(Component)]
pub struct ContractRejectButton;

#[derive(Component)]
pub struct ViewSinkButton;

#[derive(Component)]
pub struct ContractEntityLink(Entity);

#[derive(Component)]
pub struct ContractsSidebarRoot;

/// Marker component for data icons that need to be resized without replacing their Node component
#[derive(Component)]
pub struct NeedsContractResize;

/// Component to store dataset information for tooltip display
#[derive(Component, Clone)]
pub struct DatasetTooltip {
    pub dataset: Dataset,
}

fn get_contract_sort_priority(status: &ContractStatus, fulfillment: &ContractFulfillment) -> i32 {
    match status {
        ContractStatus::Active => match fulfillment.status {
            ContractFulfillmentStatus::Failing => 0,    // First
            ContractFulfillmentStatus::Meeting => 2,    // Third
            ContractFulfillmentStatus::Exceeding => 3,  // Fourth
        },
        ContractStatus::Pending => 1,                  // Second
        _ => 4,                                        // Last
    }
}

const LINE_HEIGHT: f32 = 21.;

/// Injects scroll events into the UI hierarchy.
pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

pub fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}

pub fn spawn_contracts_sidebar_ui(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Right sidebar root node
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            left: Val::Auto,
            top: Val::Px(45.0), // Start below the newsfeed (which is 64px tall)
            bottom: Val::Percent(15.0), // Stop above the bottom bar (12% height)
            width: Val::Vw(25.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            overflow: Overflow::scroll_y(),
            align_self: AlignSelf::Stretch,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
        ContractsSidebarRoot,
        BlocksWorldScroll
    ));
    
    // Spawn tooltip that will be shown on hover
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Vw(0.8)),
            display: Display::None, // Hidden by default
            max_width: Val::Vw(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
        ZIndex(1000), // High z-index to appear above everything
        GlobalZIndex(1000),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            game_assets.text_font(14.0),
            ScalableText::from_vw(1.2),
            TextColor(Color::WHITE),
            DatasetTooltipText,
        ));
    });
}

pub fn update_contracts_sidebar_ui(
    mut commands: Commands,
    sidebar_query: Query<Entity, With<ContractsSidebarRoot>>,
    contract_query: Query<(Entity, &Contract, &ContractStatus, &ContractDescription, &ContractFulfillment, &Dataset)>,
    children_query: Query<&Children>,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    let Ok(sidebar) = sidebar_query.single() else { return; };

    // Remove all children (cards) from sidebar before re-adding
    if let Ok(children) = children_query.get(sidebar) {
        let children_vec: Vec<Entity> = children.to_vec();
        for child in children_vec {
            commands.entity(child).despawn();
        }
    }

    // Collect and sort contracts by priority
    let mut contracts: Vec<_> = contract_query.iter()
        .filter(|(_, _, status, _, _, _)| matches!(status, ContractStatus::Pending | ContractStatus::Active))
        .collect();
    contracts.sort_by_key(|(_, _, status, _, fulfillment, _)| get_contract_sort_priority(status, fulfillment));

    // Add a card for each sorted contract
    for (contract_entity, _contract, status, desc, fulfillment, dataset) in contracts {
        if matches!(status, ContractStatus::Pending | ContractStatus::Active) {
            // Card background color
            let card_color = match status {
                ContractStatus::Pending => Color::srgb(0.25, 0.22, 0.10), // gold-brown for pending
                ContractStatus::Active => match fulfillment.status {
                    ContractFulfillmentStatus::Exceeding => Color::srgb(0.18, 0.32, 0.60), // blue for exceeding
                    ContractFulfillmentStatus::Meeting => Color::srgb(0.18, 0.45, 0.18),   // green for meeting
                    ContractFulfillmentStatus::Failing => Color::srgb(0.45, 0.18, 0.18),   // red for failing
                },
                _ => Color::srgb(0.15, 0.15, 0.18),
            };
            // Status text color
            let status_text_color = match status {
                ContractStatus::Pending => Color::srgb(0.95, 0.85, 0.25), // yellow for pending
                ContractStatus::Active => match fulfillment.status {
                    ContractFulfillmentStatus::Exceeding => Color::srgb(0.45, 0.65, 1.0), // light blue
                    ContractFulfillmentStatus::Meeting => Color::srgb(0.3, 0.9, 0.3),     // bright green
                    ContractFulfillmentStatus::Failing => Color::srgb(1.0, 0.3, 0.3),     // bright red
                },
                _ => Color::WHITE,
            };
            
            // First, spawn all data type icons with augmentation effects BEFORE creating the card
            let mut data_types: Vec<_> = dataset.contents.keys().cloned().collect();
            data_types.sort();
            
            let mut data_icon_entities = Vec::new();
            let mut aug_indicator_entities = Vec::new();
            use crate::factory::source_visuals::spawn_data_type_with_augmentations;
            
            for (index, data_type) in data_types.iter().enumerate() {
                if let Some(attributes) = dataset.contents.get(data_type) {
                    // Calculate position for this icon
                    let x_offset = index as f32 * 22.0;
                    let position = Vec3::new(x_offset, 0.0, 0.0);
                    
                    // This function handles BOTH augmented indicator AND scanning flash effect!
                    let (icon_entity, aug_entity) = spawn_data_type_with_augmentations(
                        &mut commands,
                        *data_type,
                        attributes.clone(),
                        position,
                        true, // is_ui = true
                        &game_assets,
                        &asset_server,
                    );
                    
                    // Mark this icon for resizing - we'll resize it after spawn without replacing Node
                    commands.entity(icon_entity).insert(NeedsContractResize);
                    
                    data_icon_entities.push(icon_entity);
                    
                    // Collect augmented indicator entities if they exist
                    if let Some(aug_indicator) = aug_entity {
                        aug_indicator_entities.push((icon_entity, aug_indicator));
                    }
                }
            }
            
            // Now create the card and add the icons to it
            let card = commands.spawn((
                Node {
                    margin: UiRect::new(Val::Vw(0.3), Val::Vw(0.3), Val::Vw(0.15), Val::Vw(0.15)),
                    padding: UiRect::all(Val::Vw(1.2)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    width: Val::Percent(100.0), // take full width of sidebar
                    position_type: PositionType::Relative,
                    ..default()
                },
                BackgroundColor(card_color),
            ))
            .with_children(|parent| {
                
                if let ContractStatus::Active = status {
                    // Create a horizontal container for the title and view sink button
                    parent.spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            width: Val::Percent(100.0),
                            margin: UiRect::bottom(Val::Vw(0.6)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    )).with_children(|header| {
                        // Left side: icons + contract name
                        header.spawn((
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Vw(0.4),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        )).with_children(|left_container| {
                            // Dataset icons container
                            let dataset_container = left_container.spawn((
                                Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Vw(0.2),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                                Interaction::None,
                                DatasetTooltip {
                                    dataset: dataset.clone(),
                                },
                            )).id();
                            
                            // Add all the pre-spawned data icons as children
                            for icon_entity in &data_icon_entities {
                                left_container.commands().entity(dataset_container).add_child(*icon_entity);
                            }
                            
                            // Add augmented indicators as children of their respective icons
                            for (icon_entity, aug_indicator) in &aug_indicator_entities {
                                left_container.commands().entity(*icon_entity).add_child(*aug_indicator);
                            }
                            
                            // Contract name
                            left_container.spawn((
                                Text::new(&desc.name),
                                game_assets.text_font(20.0),
                                ScalableText::from_vw(2.2),
                                TextColor(Color::WHITE),
                                Node { ..default() },
                            ));
                        });
                        
                        // View Sink button on the right
                        header.spawn((
                            Node {
                                padding: UiRect::all(Val::Vw(0.45)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            ViewSinkButton,
                            ContractEntityLink(contract_entity),
                            Interaction::None,
                        )).with_children(|button| {
                            button.spawn((
                                Text::new("View Sink"),
                                game_assets.text_font(12.0),
                                ScalableText::from_vw(1.5),
                                TextColor(Color::WHITE),
                                Node::default()
                            ));
                        });
                    });
                } else {
                    // Pending contracts: icons + name in horizontal row
                    parent.spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Vw(0.4),
                            margin: UiRect::bottom(Val::Vw(0.6)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    )).with_children(|left_container| {
                        // Dataset icons container
                        let dataset_container = left_container.spawn((
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Vw(0.2),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            Interaction::None,
                            DatasetTooltip {
                                dataset: dataset.clone(),
                            },
                        )).id();
                        
                        // Add all the pre-spawned data icons as children
                        for icon_entity in &data_icon_entities {
                            left_container.commands().entity(dataset_container).add_child(*icon_entity);
                        }
                        
                        // Add augmented indicators as children of their respective icons
                        for (icon_entity, aug_indicator) in &aug_indicator_entities {
                            left_container.commands().entity(*icon_entity).add_child(*aug_indicator);
                        }
                        
                        // Contract name
                        left_container.spawn((
                            Text::new(&desc.name),
                            game_assets.text_font(20.0),
                            ScalableText::from_vw(2.2),
                            TextColor(Color::WHITE),
                            Node { ..default() },
                        ));
                    });
                }
                parent.spawn((
                    Text::new(format!("Status: {:?}", status)),
                    game_assets.text_font(12.0),
                    ScalableText::from_vw(1.5),
                    TextColor(status_text_color),
                    Node { ..default() },
                ));
                if let ContractStatus::Active = status {
                    parent.spawn((
                        Text::new(format!("Fulfillment: {:?}", fulfillment.status)),
                        game_assets.text_font(12.0),
                        ScalableText::from_vw(1.5),
                        TextColor(status_text_color),
                        Node { ..default() },
                    ));

                    // Add base money and throughput info
                    parent.spawn((
                        Text::new(format!(
                            "Base income: {:.2} | Required: {:.2}",
                            fulfillment.base_money, fulfillment.base_threshold
                        )),
                        game_assets.text_font(12.0),
                        ScalableText::from_vw(0.7),
                        TextColor(Color::WHITE),
                        Node { ..default() },
                    ));

                    // Add current money and throughput info
                    parent.spawn((
                        Text::new(format!(
                            "Income: {:.2} | Throughput: {:.2}",
                            fulfillment.get_income(), fulfillment.throughput
                        )),
                        game_assets.text_font(12.0),
                        ScalableText::from_vw(1.5),
                        TextColor(Color::WHITE),
                        Node { ..default() },
                    ));

                    // Progress bar for throughput over threshold
                    let progress = (fulfillment.throughput / (fulfillment.base_threshold * 2.0)).min(1.0).max(0.0);
                    parent.spawn((
                        Node {
                            width: Val::Vw(13.5),
                            height: Val::Vh(1.5),
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.18, 0.18)),
                    ))
                    .with_children(|bar| {
                        // Progress fill
                        bar.spawn((
                            Node {
                                width: Val::Vw(13.5 * progress as f32),
                                height: Val::Vh(1.5),
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
                        ));
                        
                        // Threshold line
                        bar.spawn((
                            Node {
                                width: Val::Px(1.0),
                                height: Val::Vh(1.5),
                                position_type: PositionType::Absolute,
                                left: Val::Vw(6.75), // 50% of 13.5vw
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1., 1., 1., 0.4)), // Semi-transparent white
                        ));
                    });
                } else {
                    // Add base money and throughput info
                    parent.spawn((
                        Text::new(format!(
                            "Base income: {:.2} | Required: {:.2}",
                            fulfillment.base_money, fulfillment.base_threshold
                        )),
                        game_assets.text_font(12.0),
                        ScalableText::from_vw(1.5),
                        TextColor(Color::WHITE),
                        Node { ..default() },
                    ));

                    // Add accept/reject buttons
                    parent.spawn((
                        Node {
                            margin: UiRect::top(Val::Vw(0.6)),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    )).with_children(|buttons| {
                        // Accept button
                        buttons.spawn((
                            Node {
                                padding: UiRect::all(Val::Vw(0.6)),
                                margin: UiRect::right(Val::Vw(0.6)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                            ContractAcceptButton,
                            ContractEntityLink(contract_entity),
                            Interaction::None,
                        )).with_children(|button| {
                            button.spawn((
                                Text::new("Y"),
                                game_assets.text_font(16.0),
                                ScalableText::from_vw(2.0),
                                TextColor(Color::WHITE),
                                Node::default()
                            ));
                        });

                        // View Sink button
                        buttons.spawn((
                            Node {
                                padding: UiRect::all(Val::Vw(0.6)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            ViewSinkButton,
                            ContractEntityLink(contract_entity),
                            Interaction::None,
                        )).with_children(|button| {
                            button.spawn((
                                Text::new("View Sink"),
                                game_assets.text_font(14.0),
                                ScalableText::from_vw(1.8),
                                TextColor(Color::WHITE),
                                Node::default()
                            ));
                        });

                        // Reject button
                        buttons.spawn((
                            Node {
                                padding: UiRect::all(Val::Vw(0.6)),
                                margin: UiRect::right(Val::Vw(0.6)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                            ContractRejectButton,
                            ContractEntityLink(contract_entity),
                            Interaction::None,
                        )).with_children(|button| {
                            button.spawn((
                                Text::new("N"),
                                game_assets.text_font(14.0),
                                ScalableText::from_vw(2.0),
                                TextColor(Color::WHITE),
                                Node::default()
                            ));
                        });
                    });
                }
            })
            .id();
            commands.entity(sidebar).add_child(card);
        }
    }
}

pub fn handle_contract_buttons(
    mut contract_query: Query<&mut ContractStatus>,
    accept_query: Query<(&Interaction, &ContractEntityLink), (Changed<Interaction>, With<ContractAcceptButton>)>,
    reject_query: Query<(&Interaction, &ContractEntityLink), (Changed<Interaction>, With<ContractRejectButton>)>,
    view_sink_query: Query<(&Interaction, &ContractEntityLink), (Changed<Interaction>, With<ViewSinkButton>)>,
    associated_sink_query: Query<&AssociatedWithSink>,
    camera_query: Single<(&mut Transform, &mut Projection), With<Camera>>,
    sink_query: Query<&GridPosition, With<SinkBuilding>>, // Assuming SinkBuilding is a marker component for sink entities
    grid: Res<Grid>,
) {
    // Handle accept button clicks
    for (interaction, link) in accept_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut status) = contract_query.get_mut(link.0) {
                *status = ContractStatus::Active;
            }
        }
    }

    // Handle reject button clicks
    for (interaction, link) in reject_query.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(mut status) = contract_query.get_mut(link.0) {
                *status = ContractStatus::Rejected;
            }
        }
    }

    let (mut camera_transform, camera_projection) = camera_query.into_inner();

    // Handle view sink button clicks
    // a lot of hard coded stuff and a bit sus but it works for now
    if let Projection::Orthographic(ref mut orthographic) = *camera_projection.into_inner() {
        for (interaction, link) in view_sink_query.iter() {
            if *interaction == Interaction::Pressed {
                if let Ok(associated_sink) = associated_sink_query.get(link.0) {
                    if let Ok(sink_gridpos) = sink_query.get(associated_sink.0) {
                        // Move camera to sink grid position
                        focus_camera_on_grid_pos(sink_gridpos, &grid, &mut camera_transform, orthographic);
                    }
                }
            }
        }
    }
}

/// System to resize data icons in contracts without replacing their Node component
/// This is crucial because replacing Node breaks the ScanningFlashEffect overlay system
pub fn resize_contract_data_icons(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Node), With<NeedsContractResize>>,
) {
    for (entity, mut node) in query.iter_mut() {
        // Modify Node fields directly instead of replacing the component
        // This preserves the Children relationship needed for flash overlay system
        node.width = Val::Vw(1.5);  // Scale with viewport
        node.height = Val::Vw(1.5); // Scale with viewport
        node.position_type = PositionType::Relative; // Critical: change from Absolute to Relative for flex layout
        node.left = Val::Auto; // Clear absolute positioning
        node.top = Val::Auto;
        
        commands.entity(entity).remove::<NeedsContractResize>();
    }
}

/// System to show dataset tooltip on hover
pub fn show_dataset_tooltip(
    mut tooltip_query: Query<(&Interaction, &DatasetTooltip), Changed<Interaction>>,
    mut tooltip_text_query: Query<&mut Text, With<DatasetTooltipText>>,
) {
    for (interaction, dataset_tooltip) in tooltip_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                // Build detailed description
                let mut description = String::from("Dataset:\n");
                
                let mut data_types: Vec<_> = dataset_tooltip.dataset.contents.iter().collect();
                data_types.sort_by_key(|(dt, _)| *dt);
                
                for (data_type, attributes) in data_types {
                    description.push_str(&format!("  • {:?}", data_type));
                    
                    if !attributes.is_empty() {
                        description.push_str(" (");
                        let mut attrs: Vec<_> = attributes.iter().collect();
                        attrs.sort();
                        let attr_names: Vec<String> = attrs.iter().map(|a| format!("{:?}", a)).collect();
                        description.push_str(&attr_names.join(", "));
                        description.push_str(")");
                    }
                    description.push_str("\n");
                }
                
                // Update tooltip text
                if let Ok(mut text) = tooltip_text_query.single_mut() {
                    **text = description;
                }
            }
            _ => {}
        }
    }
}

#[derive(Component)]
pub struct DatasetTooltipText;

