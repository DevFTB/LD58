use crate::factory::physical::remove_physical_link_on_right_click;
use crate::ui::shop::clear_selection;
use crate::{assets::GameAssets, ui::tooltip::TooltipPlugin};
use crate::player::Player;
use bevy::{color::palettes::css::BROWN, prelude::*};

pub mod contracts;
pub mod interactive_event;
pub mod newsfeed;
pub mod shop;
pub mod tooltip;
pub mod money;

pub mod interaction;

pub struct UIPlugin;

pub const RIGHT_BAR_WIDTH_PCT: f32 = 20.0;

/// Marker component for UI elements that should block world clicks
#[derive(Component)]
#[require(Interaction)]
pub struct BlocksWorldClicks;

/// Marker component for the paused indicator
#[derive(Component)]
pub struct PausedIndicator;

/// Component to track fade animation for paused indicator
#[derive(Component)]
pub struct PausedFadeAnimation {
    timer: f32,
    cycle_duration: f32,
}

/// Marker component for UI elements that should block world clicks
#[derive(Component)]
#[require(Interaction)]
pub struct BlocksWorldScroll;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        use crate::pause::GameState;
        
        app.insert_resource(shop::SelectedBuildingType(None))
            .insert_resource(newsfeed::RecentNewsIds::new(5))
            .insert_resource(interactive_event::ModalSpawnCooldown::default())
            .insert_resource(interactive_event::QueuedEvents::default())
            .add_systems(
                Update,
                (
                    contracts::send_scroll_events,
                    contracts::handle_contract_buttons,
                    contracts::update_contracts_sidebar_ui,
                )
                    .chain(),
            )
            .add_observer(contracts::on_scroll_handler)
            .add_systems(Startup, spawn_paused_indicator)
            .add_systems(Startup, shop::spawn_building_shop)
            .add_systems(Startup, newsfeed::spawn_newsfeed_ui)
            .add_systems(Startup, contracts::spawn_contracts_sidebar_ui)
            .add_systems(Startup, money::spawn_money_display_ui)
            .add_systems(Update, money::update_money_display.run_if(resource_changed::<Player>))
            .add_systems(Update, (update_paused_indicator, animate_paused_fade))
            // Shop systems should work in Running and ManualPause (allow building placement while paused)
            .add_systems(Update, (
                shop::handle_building_click,
                shop::update_selected_building_position,
                shop::handle_placement_click,
                shop::handle_building_rotate,
            ).run_if(in_state(GameState::Running).or(in_state(GameState::ManualPause))))
            // Newsfeed only during gameplay
            .add_systems(Update, (
                newsfeed::generate_news,
                newsfeed::add_newsfeed_item_system,
                newsfeed::scroll_newsfeed_items,
            ).run_if(in_state(GameState::Running)))
            // Event routing and bubbles work in all non-modal states
            .add_systems(Update, (
                interactive_event::route_events_by_urgency,
                interactive_event::manage_event_bubbles,
                interactive_event::handle_bubble_clicks,
                interactive_event::animate_bubble_wobble,
            ).run_if(in_state(GameState::Running).or(in_state(GameState::ManualPause))))
            // Modal interaction always runs (needed when modal is open)
            .add_systems(
                Update,
                (
                    interactive_event::handle_choice_button_interaction,
                    interactive_event::handle_choice_click,
                    interactive_event::handle_choice_tooltip,
                    interactive_event::scale_text_system,
                ),
            )
            // Test trigger should work in Running and ManualPause
            .add_systems(Update, interactive_event::test_trigger_random_event
                .run_if(in_state(GameState::Running).or(in_state(GameState::ManualPause))))
            .add_systems(
                Update,
                clear_selection.before(remove_physical_link_on_right_click),
            );
        app.add_plugins(TooltipPlugin);
    }
}


fn spawn_paused_indicator(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Vw(8.0), // Use viewport width for responsive sizing
            display: Display::None, // Hidden by default
            position_type: PositionType::Absolute,
            bottom: Val::Percent(shop::BUILDING_BAR_HEIGHT_PCT + 10.0),
            left: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ZIndex(100),
        PausedIndicator,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Paused"),
            game_assets.text_font(80.0), // Use game font at 80px (will be overridden by ScalableText)
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)), // Start at 50% opacity
            TextLayout::new_with_justify(Justify::Center),
            interactive_event::ScalableText::from_vw(5.0), // 5vw font size
            PausedFadeAnimation {
                timer: 0.0,
                cycle_duration: 2.0, // 2 second fade cycle (1s in, 1s out)
            },
        ));
    });
}

fn update_paused_indicator(
    state: Res<State<crate::pause::GameState>>,
    mut query: Query<&mut Node, With<PausedIndicator>>,
) {
    use crate::pause::GameState;
    
    if let Ok(mut node) = query.single_mut() {
        node.display = if *state.get() == GameState::ManualPause {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn animate_paused_fade(
    time: Res<Time>,
    state: Res<State<crate::pause::GameState>>,
    mut query: Query<(&mut PausedFadeAnimation, &mut TextColor)>,
) {
    use crate::pause::GameState;
    
    // Only animate when paused
    if *state.get() != GameState::ManualPause {
        // Reset timer when not paused
        for (mut anim, mut color) in &mut query {
            anim.timer = 0.0;
            color.0.set_alpha(0.5); // Reset to 50% opacity
        }
        return;
    }
    
    for (mut anim, mut color) in &mut query {
        anim.timer += time.delta_secs();
        
        // Loop the animation
        if anim.timer >= anim.cycle_duration {
            anim.timer -= anim.cycle_duration;
        }
        
        // Calculate fade (0.5 to 0.9 and back) - never fully transparent
        let progress = anim.timer / anim.cycle_duration;
        let alpha = if progress < 0.5 {
            // Fade in (0 to 1)
            progress * 2.0
        } else {
            // Fade out (1 to 0)
            2.0 - (progress * 2.0)
        };
        
        // Apply smooth easing (ease in-out)
        let eased_alpha = alpha * alpha * (3.0 - 2.0 * alpha);
        
        // Scale from 0.5 (50%) to 0.9 (90%) opacity
        color.0.set_alpha(0.5 + (eased_alpha * 0.4));
    }
}
