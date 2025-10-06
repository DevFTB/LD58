use bevy::prelude::*;
use crate::factions::{Faction, reputation_score_to_level};
use crate::events::InteractiveEventLibrary;
use serde::Deserialize;
use rand::prelude::IndexedRandom;

/// Types of consequences for event choices.
#[derive(Debug, Clone, Deserialize)]
pub enum ConsequenceType {
    Money(i32), // Positive for gain, negative for loss
    Reputation(Faction, i32), // Faction and amount
    GameOver, // Triggers game over
    UnlockFactionEvent(u32),
    AcceptContract(u32),
    Bankruptcy(u32)
}

/// Represents a consequence of a choice.
#[derive(Debug, Clone, Deserialize)]
pub struct EventConsequence {
    pub consequence_type: ConsequenceType,
}

/// Represents a choice in an interactive event.
#[derive(Debug, Clone, Deserialize)]
pub struct EventChoice {
    pub description: String,
    pub consequences: Vec<EventConsequence>,
    pub chance: Option<Vec<f32>>
}

/// An interactive event item from the event library
#[derive(Debug, Clone, Deserialize)]
pub struct InteractiveEventItem {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub choices: Vec<EventChoice>,
    pub weight: Option<u32>,
    pub repeatable: Option<bool>
}

/// Data for an interactive event to be displayed
#[derive(Debug, Clone)]
pub struct InteractiveEventData {
    pub title: String,
    pub description: String,
    pub faction: Faction,
    pub choices: Vec<EventChoice>,
    pub repeatable: Option<bool>,
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

/// Get a random interactive event based on faction and reputation level.
/// Excludes recently used event IDs.
/// If all items are exhausted, drops the oldest ID (FIFO) and retries.
/// Returns None if event data hasn't loaded yet or if no suitable event is found.
pub fn get_interactive_faction_event(
    faction: Faction,
    rep: u32,
    event_library: &InteractiveEventLibrary,
    recent_ids: &mut Vec<u32>,
) -> Option<InteractiveEventData> {
    let rep_level = reputation_score_to_level(rep);
    
    // Get the faction's data
    let faction_data = event_library.0.get(&faction)?;
    
    // Get items for this reputation level
    let items = faction_data.get(&rep_level)?;
    
    // Filter out recently used IDs
    let available_items: Vec<&InteractiveEventItem> = items
        .iter()
        .filter(|item| !recent_ids.contains(&item.id))
        .collect();
    
    // If no items available (all recently used), drop the oldest ID (FIFO) and retry
    if available_items.is_empty() && !recent_ids.is_empty() {
        recent_ids.remove(0); // Drop oldest (FIFO)
        
        // Retry with updated recent_ids
        let available_items: Vec<&InteractiveEventItem> = items
            .iter()
            .filter(|item| !recent_ids.contains(&item.id))
            .collect();
        
        let mut rng = rand::rng();
        return available_items.choose(&mut rng).map(|item| InteractiveEventData {
            title: item.title.clone(),
            description: item.description.clone(),
            faction,
            choices: item.choices.clone(),
        });
    }
    
    // Select a random item
    let mut rng = rand::rng();
    available_items.choose(&mut rng).map(|item| InteractiveEventData {
        title: item.title.clone(),
        description: item.description.clone(),
        faction,
        choices: item.choices.clone(),
    })
}
