use bevy::prelude::*;
use crate::factions::{Faction, ReputationLevel};
use std::collections::HashMap;
use serde::Deserialize;

pub mod event_data;


#[derive(Resource, Deserialize, Debug)]
pub struct NewsLibrary(pub HashMap<Faction, HashMap<ReputationLevel, Vec<NewsItem>>>);


#[derive(Debug, Clone, Deserialize)]
pub struct NewsItem {
    pub id: u32,
    pub text: String,
}

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

/// Types of consequences for event choices.
#[derive(Debug, Clone)]
pub enum ConsequenceType {
    Money(i32), // Positive for gain, negative for loss
    Reputation(Faction, i32), // Faction and amount (positive increase, negative decrease)
    // Add more as needed
}

/// Represents a choice in an interactive event.
#[derive(Debug, Clone)]
pub struct EventChoice {
    pub description: String,
    pub consequences: Vec<EventConsequence>,
}

/// Represents a consequence of a choice.
#[derive(Debug, Clone)]
pub struct EventConsequence {
    pub consequence_type: ConsequenceType,
}

/// Data for an interactive event.
#[derive(Debug, Clone)]
pub struct InteractiveEventData {
    pub title: String,
    pub description: String,
    pub faction: Faction,
    pub choices: Vec<EventChoice>,
}

/// Bevy event to show an interactive event popup.
#[derive(Event, Message)]
pub struct ShowInteractiveEvent {
    pub event_data: InteractiveEventData,
}

/// Bevy event sent when player makes a choice in interactive event.
#[derive(Event, Message)]
pub struct PlayerChoiceEvent {
    pub choice_data: EventChoice,
}

/// Bevy event to add an item to the newsfeed.
#[derive(Event, Message)]
pub struct AddNewsfeedItemEvent {
    pub faction: Faction, // Optional, for faction-specific items
    pub headline: String,
}

/// Plugin for events system.
pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShowInteractiveEvent>()
            .add_message::<PlayerChoiceEvent>()
            .add_message::<AddNewsfeedItemEvent>()
            .add_systems(PreStartup, load_news_events_from_ron);
    }
}