use bevy::prelude::*;
use crate::contracts::{Contract, ContractStatus, ContractDescription};

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
    contract_query: Query<(&Contract, &ContractStatus, &ContractDescription)>,
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
    for (_contract, status, desc) in contract_query.iter() {
        if matches!(status, ContractStatus::Pending | ContractStatus::Active) {
            let color = match status {
                ContractStatus::Active => Color::srgb(0.2, 0.7, 0.3),
                ContractStatus::Pending => Color::srgb(0.8, 0.8, 0.2),
                _ => Color::srgb(0.5, 0.5, 0.5)
            };
            let card = commands.spawn((
                Node {
                    margin: UiRect::all(Val::Px(8.0)),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(&desc.name),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(color),
                    Node { ..default() },
                ));
                parent.spawn((
                    Text::new(format!("Status: {:?}", status)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(color),
                    Node { ..default() },
                ));
            })
            .id();
            commands.entity(sidebar).add_child(card);
        }
    }
}