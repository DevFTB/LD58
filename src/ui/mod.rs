use bevy::{
    color::palettes::css::BROWN,
    prelude::*,
};

pub mod newsfeed;
pub mod shop;
pub mod contracts;

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
            .add_systems(Update, contracts::update_contracts_sidebar_ui)        
            .add_systems(Update, contracts::send_scroll_events)
            .add_observer(contracts::on_scroll_handler);
    }
}

