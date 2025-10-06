use bevy::prelude::*;
use crate::contracts::{Contract, ContractDescription, ContractFulfillment, ContractStatus, ContractFulfillmentStatus};

#[derive(Component)]
pub struct ContractsSidebarRoot;

pub fn spawn_contracts_sidebar_ui(mut commands: Commands) {
    // Right sidebar root node
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(0.0),
            left: Val::Auto,
            top: Val::Px(0.0),
            width: Val::Px(340.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.12)),
        ContractsSidebarRoot,
    ));
}

pub fn update_contracts_sidebar_ui(
    mut commands: Commands,
    sidebar_query: Query<Entity, With<ContractsSidebarRoot>>,
    contract_query: Query<(&Contract, &ContractStatus, &ContractDescription, &ContractFulfillment)>,
    children_query: Query<&Children>,
) {
    let Ok(sidebar) = sidebar_query.single() else { return; };

    // Remove all children (cards) from sidebar before re-adding
    if let Ok(children) = children_query.get(sidebar) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // Add a card for each contract (pending or active)
    for (_contract, status, desc, fulfillment) in contract_query.iter() {
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
            let card = commands.spawn((
                Node {
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    width: Val::Percent(100.0), // take full width of sidebar
                    ..default()
                },
                BackgroundColor(card_color),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(&desc.name),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::WHITE),
                    Node { ..default() },
                ));
                parent.spawn((
                    Text::new(format!("Status: {:?}", status)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(status_text_color),
                    Node { ..default() },
                ));
                if let ContractStatus::Active = status {
                    parent.spawn((
                        Text::new(format!("Fulfillment: {:?}", fulfillment.status)),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(status_text_color),
                        Node { ..default() },
                    ));

                    // Add current money and throughput info
                    parent.spawn((
                        Text::new(format!(
                            "Income: {:.2} | Throughput: {:.2}",
                            fulfillment.get_income(), fulfillment.throughput
                        )),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::WHITE),
                        Node { ..default() },
                    ));

                    // Progress bar for throughput over threshold
                    let progress = (fulfillment.throughput / fulfillment.base_threshold).min(1.0).max(0.0);
                    parent.spawn((
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(12.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.18, 0.18)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: Val::Px(180.0 * progress as f32),
                                height: Val::Px(12.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
                        ));
                    });
                } else {
                    // Add base money and throughput info
                    parent.spawn((
                        Text::new(format!(
                            "Base income: {:.2} | Throughput: {:.2}",
                            fulfillment.base_money, fulfillment.base_threshold
                        )),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::WHITE),
                        Node { ..default() },
                    ));
                }
            })
            .id();
            commands.entity(sidebar).add_child(card);
        }
    }
}