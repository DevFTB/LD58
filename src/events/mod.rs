use bevy::prelude::*;
use crate::factions::{Faction, ReputationLevel};
use crate::player::Player;
use std::collections::HashMap;
use serde::Deserialize;

pub mod newsfeed_events;
// pub mod interactive_events; // Old version - replaced by interactive_events2
pub mod interactive_events;
pub mod event_triggers;

pub use newsfeed_events::{NewsItem, AddNewsfeedItemEvent};
pub use interactive_events::*;
pub use event_triggers::*;

#[derive(Resource, Deserialize, Debug)]
pub struct NewsLibrary(pub HashMap<Faction, HashMap<ReputationLevel, Vec<NewsItem>>>);

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

    // Parse the RON events as a Vec
    #[derive(Deserialize)]
    struct EventsFile {
        events: Vec<InteractiveEventItem>,
    }
    
    let events_file: EventsFile = ron::from_str(&ron_str)
        .expect("Failed to parse interactive events from RON");

    // Create library with pre-built indices
    let event_library = InteractiveEventLibrary::new(events_file.events);

    // Insert the fully loaded data as a Bevy Resource.
    commands.insert_resource(event_library);
    info!("Interactive events loaded and inserted as a Resource.");
}

/// Plugin for events system.
pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        use crate::pause::GameState;
        
        app.add_message::<ShowInteractiveEvent>()
            .add_message::<TriggerInteractiveEvent>()
            .add_message::<PlayerChoiceEvent>()
            .add_message::<AddNewsfeedItemEvent>()
            .init_resource::<EventState>()
            .init_resource::<Player>()
            .init_resource::<RandomEventTimer>()
            .add_systems(PreStartup, (load_news_events_from_ron, load_interactive_events_from_ron))
            // These systems should only run during normal gameplay (not paused or in modal)
            .add_systems(Update, (
                random_event_trigger_system,
                forced_event_checker_system,
                handle_manual_event_triggers,
                bankruptcy_update_system,
            ).run_if(in_state(GameState::Running)))
            .add_systems(Update, (
                handle_player_choice_system,
            ));
    }
}