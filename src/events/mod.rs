use bevy::prelude::*;
use crate::factions::{Faction, ReputationLevel};
use std::collections::HashMap;
use serde::Deserialize;

pub mod newsfeed_events;
pub mod interactive_events;

pub use newsfeed_events::{NewsItem, AddNewsfeedItemEvent};
pub use interactive_events::{
    EventChoice, InteractiveEventItem, InteractiveEventData, 
    ShowInteractiveEvent, PlayerChoiceEvent
};

#[derive(Resource, Deserialize, Debug)]
pub struct NewsLibrary(pub HashMap<Faction, HashMap<ReputationLevel, Vec<NewsItem>>>);

#[derive(Resource, Deserialize, Debug)]
pub struct InteractiveEventLibrary(pub HashMap<Faction, HashMap<ReputationLevel, Vec<InteractiveEventItem>>>);

// A startup system to read the file and insert it as a resource.
fn load_news_events_from_ron(mut commands: Commands) {
    // Read the file from the assets folder.
    let ron_str = std::fs::read_to_string("assets/text/news.ron")
        .expect("Failed to read news.ron");

    // Parse the RON string into our NewsLibrary struct.
    let news_library: NewsLibrary = ron::from_str(&ron_str)
        .expect("Failed to parse news events from RON");

    // Insert the fully loaded data as a Bevy Resource.
    commands.insert_resource(news_library);
    info!("News events loaded and inserted as a Resource.");
}

// A startup system to read interactive events from RON file.
fn load_interactive_events_from_ron(mut commands: Commands) {
    // Read the file from the assets folder.
    let ron_str = std::fs::read_to_string("assets/text/interactive_events.ron")
        .expect("Failed to read interactive_events.ron");

    // Parse the RON string into our InteractiveEventLibrary struct.
    let event_library: InteractiveEventLibrary = ron::from_str(&ron_str)
        .expect("Failed to parse interactive events from RON");

    // Insert the fully loaded data as a Bevy Resource.
    commands.insert_resource(event_library);
    info!("Interactive events loaded and inserted as a Resource.");
}

/// Plugin for events system.
pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<interactive_events::ShowInteractiveEvent>()
            .add_message::<interactive_events::PlayerChoiceEvent>()
            .add_message::<newsfeed_events::AddNewsfeedItemEvent>()
            .add_systems(Startup, (load_news_events_from_ron, load_interactive_events_from_ron));
    }
}