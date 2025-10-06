use bevy::prelude::*;
use crate::factions::{Faction, reputation_score_to_level};
use crate::events::NewsLibrary;
use serde::Deserialize;
use rand::prelude::IndexedRandom;

/// A news item from the news library
#[derive(Debug, Clone, Deserialize)]
pub struct NewsItem {
    pub id: u32,
    pub text: String,
}

/// Bevy event to add an item to the newsfeed.
#[derive(Event, Message)]
pub struct AddNewsfeedItemEvent {
    pub faction: Faction,
    pub headline: String,
}


pub fn get_news_headline(
    faction: Faction,
    rep: u32,
    news_library: &NewsLibrary,
    recent_ids: &mut Vec<u32>,
) -> Option<(u32, String)> {
    let rep_level = reputation_score_to_level(rep);
    
    // Get the faction's data
    let faction_data = news_library.0.get(&faction)?;
    
    // Get items for this reputation level
    let items = faction_data.get(&rep_level)?;
    
    // Filter out recently used IDs
    let available_items: Vec<&NewsItem> = items
        .iter()
        .filter(|item| !recent_ids.contains(&item.id))
        .collect();
    
    // If no items available (all recently used), drop the oldest ID (FIFO) and retry
    if available_items.is_empty() && !recent_ids.is_empty() {
        recent_ids.remove(0); // Drop oldest (FIFO)
        
        // Retry with updated recent_ids
        let available_items: Vec<&NewsItem> = items
            .iter()
            .filter(|item| !recent_ids.contains(&item.id))
            .collect();
        
        let mut rng = rand::rng();
        return available_items.choose(&mut rng).map(|item| (item.id, item.text.clone()));
    }
    
    // Select a random item
    let mut rng = rand::rng();
    available_items.choose(&mut rng).map(|item| (item.id, item.text.clone()))
}
