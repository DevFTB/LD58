use bevy::prelude::*;
use crate::player::Player;

#[derive(Component)]
pub struct MoneyDisplay;

/// Spawns the money display UI in the top left corner
pub fn spawn_money_display_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), // Semi-transparent black background
        ZIndex(100), // Make sure it's on top
        MoneyDisplay,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("$0"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.1)), // Gold color for money
            Node::default(),
        ));
    });
}

/// Updates the money display text when player money changes
pub fn update_money_display(
    player: Res<Player>,
    money_display_query: Query<&Children, With<MoneyDisplay>>,
    mut text_query: Query<&mut Text>,
) {
    for children in money_display_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                // Format money with commas for readability
                let formatted_money = format!("${}", format_number_with_commas(player.money));
                **text = formatted_money;
            }
        }
    }
}

/// Helper function to format numbers with commas (e.g., 1000 -> "1,000")
fn format_number_with_commas(mut num: i32) -> String {
    if num == 0 {
        return "0".to_string();
    }
    
    let negative = num < 0;
    if negative {
        num = -num;
    }
    
    let num_str = num.to_string();
    let mut result = String::new();
    
    for (i, digit) in num_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(digit);
    }
    
    if negative {
        result.push('-');
    }
    
    result.chars().rev().collect()
}