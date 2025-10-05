use bevy::{
    color::palettes::css::BROWN,
    prelude::*,
};

pub mod newsfeed;
pub mod shop;

pub struct UIPlugin;

pub const RIGHT_BAR_WIDTH_PCT: f32 = 20.0;

/// Marker component for UI elements that should block world clicks
#[derive(Component)]
#[require(Interaction)]
pub struct BlocksWorldClicks;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(shop::SelectedBuildingType(None))
            .insert_resource(newsfeed::RecentNewsIds::new(5))
            .add_systems(Startup, startup)
            .add_systems(Startup, shop::spawn_building_shop)
            .add_systems(Startup, newsfeed::spawn_newsfeed_ui)
            .add_systems(Update, shop::handle_building_click)
            .add_systems(Update, shop::update_selected_building_position)
            .add_systems(Update, shop::handle_placement_click)
            .add_systems(Update, shop::handle_building_rotate)
            .add_systems(Update, newsfeed::add_newsfeed_item_system)
            .add_systems(Update, newsfeed::scroll_newsfeed_items)
            .add_systems(Update, newsfeed::generate_news);
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


