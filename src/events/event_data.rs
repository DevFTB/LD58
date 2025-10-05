use crate::events::{InteractiveEventData, EventChoice, EventConsequence, ConsequenceType, NewsLibrary, NewsItem};
use crate::factions::{Faction, reputation_score_to_level};
use bevy::prelude::*;
use rand::prelude::IndexedRandom;

/// Generate a news headline based on faction and reputation level.
/// Excludes recently used event IDs.
/// If all items are exhausted, drops the oldest ID (FIFO) and retries.
/// Returns None if news data hasn't loaded yet or if no suitable headline is found.
pub fn get_news_headline<'a>(
    faction: Faction,
    rep: u32,
    news_library: &'a NewsLibrary,
    recent_ids: &mut Vec<u32>,
) -> Option<(u32, String)> {
    // Wait for news data to load
    let faction_key = (faction as u8).to_string();
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

/// Placeholder event data for testing.
/// This would typically be loaded from a file or asset.
pub fn get_sample_interactive_events() -> Vec<InteractiveEventData> {
    vec![
        InteractiveEventData {
            title: "Corporate Data Breach".to_string(),
            description: "A major corporate client has suffered a data breach. They are demanding compensation or threatening to cut ties.".to_string(),
            faction: Faction::Corporate,
            choices: vec![
                EventChoice {
                    description: "Pay compensation to maintain relationship.".to_string(),
                    consequences: vec![
                        EventConsequence {
                            consequence_type: ConsequenceType::Money(-1000),
                        },
                        EventConsequence {
                            consequence_type: ConsequenceType::Reputation(Faction::Corporate, 5),
                        },
                    ],
                },
                EventChoice {
                    description: "Refuse and risk losing the contract.".to_string(),
                    consequences: vec![
                        EventConsequence {
                            consequence_type: ConsequenceType::Reputation(Faction::Corporate, -10),
                        },
                    ],
                },
            ],
        },
        InteractiveEventData {
            title: "Academic Research Opportunity".to_string(),
            description: "Academia offers a lucrative research contract, but it requires de-identified data.".to_string(),
            faction: Faction::Academia,
            choices: vec![
                EventChoice {
                    description: "Accept and provide de-identified data.".to_string(),
                    consequences: vec![
                        EventConsequence {
                            consequence_type: ConsequenceType::Money(500),
                        },
                        EventConsequence {
                            consequence_type: ConsequenceType::Reputation(Faction::Academia, 5),
                        },
                    ],
                },
                EventChoice {
                    description: "Decline to avoid ethical concerns.".to_string(),
                    consequences: vec![
                        EventConsequence {
                            consequence_type: ConsequenceType::Reputation(Faction::Academia, -5),
                        },
                    ],
                },
            ],
        },
    ]
}


