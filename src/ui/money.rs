use bevy::prelude::*;
use crate::player::Player;

#[derive(Component)]
pub struct MoneyDisplay;

#[derive(Component)]
pub struct MoneyText;

#[derive(Component)]
pub struct IncomeText;

/// Spawns the money display UI in the top left corner
pub fn spawn_money_display_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), // Semi-transparent black background
        ZIndex(100), // Make sure it's on top
        MoneyDisplay,
    ))
    .with_children(|parent| {
        // Money display
        parent.spawn((
            Text::new("$0"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.1)), // Gold color for money
            Node::default(),
            MoneyText,
        ));
        
        // Income display
        parent.spawn((
            Text::new("Income: $0/s"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.9, 0.7)), // Light green for income
            Node {
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            },
            IncomeText,
        ));
    });
}

/// Updates the money display text when player money changes
pub fn update_money_display(
    player: Res<Player>,
    mut money_text_query: Query<&mut Text, With<MoneyText>>,
    mut income_query: Query<(&mut Text, &mut TextColor), (With<IncomeText>, Without<MoneyText>)>,
) {
    // Update money display
    for mut text in money_text_query.iter_mut() {
        let formatted_money = format!("${}", format_number_with_commas(player.money));
        **text = formatted_money;
    }
    
    // Update income display with dynamic color
    for (mut text, mut color) in income_query.iter_mut() {
        let income_prefix = if player.net_income > 0 { 
            "+" 
        } else if player.net_income < 0 { 
            "" 
        } else { 
            "" 
        };
        
        let formatted_income = format!("{}${}/s", 
            income_prefix, 
            format_number_with_commas(player.net_income));
        **text = formatted_income;
        
        // Set color based on income: green for positive, red for negative, gray for zero
        *color = if player.net_income > 0 {
            TextColor(Color::srgb(0.7, 0.9, 0.7)) // Light green
        } else if player.net_income < 0 {
            TextColor(Color::srgb(0.9, 0.5, 0.5)) // Light red
        } else {
            TextColor(Color::srgb(0.7, 0.7, 0.7)) // Gray
        };
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