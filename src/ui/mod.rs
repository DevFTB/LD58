use bevy::{color::palettes::css::BROWN, prelude::*};

pub mod newsfeed;
pub mod shop;
pub mod contracts;
pub mod interactive_event;
pub mod tooltip;

pub struct UIPlugin;

pub const RIGHT_BAR_WIDTH_PCT: f32 = 20.0;

/// Marker component for UI elements that should block world clicks
#[derive(Component)]
#[require(Interaction)]
pub struct BlocksWorldClicks;

/// Marker component for UI elements that should block world clicks
#[derive(Component)]
#[require(Interaction)]
pub struct BlocksWorldScroll;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(shop::SelectedBuildingType(None))
            .insert_resource(newsfeed::RecentNewsIds::new(5))
            .insert_resource(interactive_event::ModalSpawnCooldown::default())
            .insert_resource(interactive_event::QueuedEvents::default())
            .add_systems(Update, (
                contracts::send_scroll_events,
                contracts::handle_contract_buttons,
                contracts::update_contracts_sidebar_ui,
            ).chain())
            .add_observer(contracts::on_scroll_handler)
            .add_systems(Startup, startup)
            .add_systems(Startup, shop::spawn_building_shop)
            .add_systems(Startup, newsfeed::spawn_newsfeed_ui)
            .add_systems(Startup, contracts::spawn_contracts_sidebar_ui)
            .add_systems(Update, shop::handle_building_click)
            .add_systems(Update, shop::update_selected_building_position)
            .add_systems(Update, shop::handle_placement_click)
            .add_systems(Update, shop::handle_building_rotate)
            .add_systems(Update, newsfeed::add_newsfeed_item_system)
            .add_systems(Update, newsfeed::scroll_newsfeed_items)
            .add_systems(Update, newsfeed::generate_news)
            .add_systems(Update, interactive_event::route_events_by_urgency)
            .add_systems(Update, interactive_event::manage_event_bubbles)
            .add_systems(Update, interactive_event::handle_bubble_clicks)
            .add_systems(Update, interactive_event::animate_bubble_wobble)
            .add_systems(
                Update,
                (
                    interactive_event::handle_choice_button_interaction,
                    interactive_event::handle_choice_click,
                    interactive_event::handle_choice_tooltip,
                    interactive_event::scale_text_system,
                ),
            )
            .add_systems(Update, interactive_event::test_trigger_random_event);
        // app.add_plugins(TooltipPlugin);
    }
}

fn startup(mut commands: Commands) {
    // spawn the right bar with other information: contracts + newsfeed atm
    commands.spawn((
        Node {
            width: Val::Percent(RIGHT_BAR_WIDTH_PCT),
            height: Val::Percent(100.0),
            display: Display::Flex,
            position_type: PositionType::Absolute,
            top: Val::Percent(0.0),
            left: Val::Percent(100.0 - RIGHT_BAR_WIDTH_PCT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            ..default()
        },
        BackgroundColor(BROWN.into()),
        ZIndex(-1),
        BlocksWorldClicks,
    ));
}
