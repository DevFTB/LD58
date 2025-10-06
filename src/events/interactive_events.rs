use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::factions::{Faction, FactionReputations, ReputationLevel};
use crate::player::Player;

pub const RANDOM_EVENT_COOLDOWN_SECONDS: f32 = 60.0; // 2 minutes

/// Requirements that must be met for an event to trigger
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Requirements {
    MinReputation { faction: Faction, reputation: ReputationLevel },
    MaxReputation { faction: Faction, reputation: ReputationLevel },
    ExactReputation { faction: Faction, reputation: ReputationLevel },
    /// Player must have at least this much money
    MinMoney(i32),
    /// Player must have at most this much money
    MaxMoney(i32),
    /// Must be at least this year
    MinYear(u32),
    /// Must be at most this year
    MaxYear(u32),
    /// Must be exactly this year
    SpecificYear(u32),
    /// All nested requirements must be met (AND)
    AllOf(Vec<Requirements>),
    /// At least one nested requirement must be met (OR)
    AnyOf(Vec<Requirements>),
    /// None of the nested requirements must be met (NOT)
    NoneOf(Vec<Requirements>),
    /// Event with this ID must be unlocked
    EventUnlocked(String),
    /// Event with this ID must NOT be completed
    EventNotCompleted(String),
    ContractFulfilled(i32)
}

/// How an event should be triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventTriggerMode {
    /// Event can be randomly selected when requirements are met
    Random { weight: f32 },
    /// Event triggers automatically when requirements are met (checked each game tick)
    Forced,
    /// Event can ONLY be triggered by explicit game system call via TriggerInteractiveEvent
    /// Requirements are still checked, but the event won't auto-trigger
    Manual,
}

/// Represents consequences of player choices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsequenceType {
    /// Unlock an event for future triggering
    UnlockEvent(String),
    /// Add or subtract money
    ModifyMoney(i32),
    /// Add or subtract reputation with a faction
    ModifyReputation { faction: Faction, amount: i32 },
    /// Mark a specific event as completed
    CompleteEvent(String),
    /// Trigger bankruptcy (game over?)
    Bankruptcy,
    UnlockContract(i32)

}

/// A single choice option within an interactive event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChoice {
    pub text: String,
    #[serde(default)]
    pub requirements: Vec<Requirements>,
    pub consequences: Vec<ConsequenceType>,
}

/// The complete interactive event item loaded from RON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveEventItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub trigger_mode: EventTriggerMode,
    pub faction: Option<Faction>,  // Optional: which faction this event relates to
    pub choices: Vec<EventChoice>,
    #[serde(default)]
    pub requirements: Vec<Requirements>,
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub popup_urgency: bool
}

/// Message sent when player makes a choice in an interactive event
#[derive(Message, Clone, Debug)]
pub struct PlayerChoiceEvent {
    pub event_id: String,
    pub choice_index: usize,
}

/// Resource containing all interactive events as a flat list
#[derive(Resource, Debug)]
pub struct InteractiveEventLibrary {
    pub events: Vec<InteractiveEventItem>,
    /// Pre-built indices for efficient lookup
    random_event_indices: Vec<usize>,
    forced_event_indices: Vec<usize>,
    manual_event_indices: Vec<usize>,
    /// Map event ID to index for quick lookup
    id_to_index: HashMap<String, usize>,
}

impl InteractiveEventLibrary {
    pub fn new(events: Vec<InteractiveEventItem>) -> Self {
        let mut library = Self {
            events,
            random_event_indices: Vec::new(),
            forced_event_indices: Vec::new(),
            manual_event_indices: Vec::new(),
            id_to_index: HashMap::new(),
        };
        library.build_indices();
        library
    }

    /// Build indices for efficient event lookup
    fn build_indices(&mut self) {
        self.random_event_indices.clear();
        self.forced_event_indices.clear();
        self.manual_event_indices.clear();
        self.id_to_index.clear();

        for (idx, event) in self.events.iter().enumerate() {
            // Build ID map
            self.id_to_index.insert(event.id.clone(), idx);

            // Build trigger mode indices
            match event.trigger_mode {
                EventTriggerMode::Random { .. } => self.random_event_indices.push(idx),
                EventTriggerMode::Forced => self.forced_event_indices.push(idx),
                EventTriggerMode::Manual => self.manual_event_indices.push(idx),
            }
        }
    }

    /// Get event by ID
    pub fn get_event_by_id(&self, event_id: &str) -> Option<&InteractiveEventItem> {
        self.id_to_index.get(event_id).map(|&idx| &self.events[idx])
    }

    /// Get all random events that meet their requirements
    pub fn get_eligible_random_events(&self, context: &GameContext, current_time: f64, queued_event_ids: &[String]) -> Vec<(usize, f32)> {
        self.random_event_indices
            .iter()
            .filter_map(|&idx| {
                let event = &self.events[idx];
                if context.check_requirements(&event.requirements, event.repeatable, &event.id)
                    && let EventTriggerMode::Random { weight } = event.trigger_mode {
                        // Check if this event was completed recently
                        if let Some(&last_time) = context.event_state.last_completion_time.get(&event.id) {
                            let time_since_completion = current_time - last_time;
                            if time_since_completion < RANDOM_EVENT_COOLDOWN_SECONDS as f64 {
                                return None; // Event is on cooldown
                            }
                        }
                        
                        // Check if this event is already in the queue (only for non-urgent events)
                        // Urgent events should always be allowed to trigger
                        if !event.popup_urgency && queued_event_ids.contains(&event.id) {
                            return None; // Event is already queued
                        }
                        
                        return Some((idx, weight));
                    }
                None
            })
            .collect()
    }

    /// Get all forced events that should trigger
    pub fn get_triggered_forced_events(&self, context: &GameContext) -> Vec<usize> {
        self.forced_event_indices
            .iter()
            .filter_map(|&idx| {
                let event = &self.events[idx];
                if context.check_requirements(&event.requirements, event.repeatable, &event.id) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a manual event can be triggered (requirements met)
    pub fn can_trigger_manual_event(&self, event_id: &str, context: &GameContext) -> bool {
        if let Some(&idx) = self.id_to_index.get(event_id) {
            let event = &self.events[idx];
            context.check_requirements(&event.requirements, event.repeatable, event_id)
        } else {
            false
        }
    }
}

/// Tracks which events have been unlocked and completed
#[derive(Resource, Debug, Default)]
pub struct EventState {
    pub unlocked_events: HashSet<String>,
    pub completed_events: HashMap<String, u32>, // event_id -> completion_count
    pub last_completion_time: HashMap<String, f64>, // event_id -> timestamp in seconds
}

impl EventState {
    pub fn is_unlocked(&self, event_id: &str) -> bool {
        self.unlocked_events.contains(event_id)
    }

    pub fn is_completed(&self, event_id: &str) -> bool {
        self.completed_events.contains_key(event_id)
    }

    pub fn completion_count(&self, event_id: &str) -> u32 {
        *self.completed_events.get(event_id).unwrap_or(&0)
    }

    pub fn unlock_event(&mut self, event_id: String) {
        self.unlocked_events.insert(event_id);
    }

    pub fn complete_event(&mut self, event_id: String, current_time: f64) {
        *self.completed_events.entry(event_id.clone()).or_insert(0) += 1;
        self.last_completion_time.insert(event_id, current_time);
    }
}

/// Context for checking event requirements
pub struct GameContext<'a> {
    pub player: &'a Player,
    pub factions: &'a FactionReputations,
    pub event_state: &'a EventState,
}

impl<'a> GameContext<'a> {
    /// Check if all requirements are met
    pub fn check_requirements(&self, requirements: &[Requirements], repeatable: bool, event_id: &str) -> bool {
        // If event is not repeatable and already completed, it can't trigger
        if !repeatable && self.event_state.is_completed(event_id) {
            return false;
        }

        requirements.iter().all(|req| self.check_requirement(req))
    }

    fn check_requirement(&self, requirement: &Requirements) -> bool {
        match requirement {
            Requirements::MinReputation { faction, reputation } => {
                self.factions.get_level(*faction) >= *reputation
            }
            Requirements::MaxReputation { faction, reputation } => {
                self.factions.get_level(*faction) <= *reputation
            }
            Requirements::ExactReputation { faction, reputation } => {
                self.factions.get_level(*faction) == *reputation
            }
            Requirements::MinMoney(amount) => self.player.money >= *amount,
            Requirements::MaxMoney(amount) => self.player.money <= *amount,
            Requirements::MinYear(year) => self.player.current_year >= *year,
            Requirements::MaxYear(year) => self.player.current_year <= *year,
            Requirements::SpecificYear(year) => self.player.current_year == *year,
            Requirements::AllOf(reqs) => reqs.iter().all(|r| self.check_requirement(r)),
            Requirements::AnyOf(reqs) => reqs.iter().any(|r| self.check_requirement(r)),
            Requirements::NoneOf(reqs) => !reqs.iter().any(|r| self.check_requirement(r)),
            Requirements::EventUnlocked(id) => self.event_state.is_unlocked(id),
            Requirements::EventNotCompleted(id) => !self.event_state.is_completed(id),
            Requirements::ContractFulfilled(_contract_id) => {
                // TODO: Implement contract checking when contract system is ready
                false // For now, always fail this requirement
            }
        }
    }
}

/// Data structure sent with ShowInteractiveEvent message
#[derive(Clone, Debug)]
pub struct InteractiveEventData {
    pub event_id: String,
    pub title: String,
    pub description: String,
    pub faction: Option<Faction>,  // Optional: which faction this event relates to
    pub choices: Vec<EventChoice>,
    pub popup_urgency: bool,  // If true, shows immediately; if false, queues as bubble
}

/// Message to show an interactive event modal (internal - triggered by systems)
#[derive(Message, Clone, Debug)]
pub struct ShowInteractiveEvent(pub InteractiveEventData);

/// Message sent by game systems to manually trigger a specific event
/// This is the public API for triggering Manual mode events
#[derive(Message, Clone, Debug)]
pub struct TriggerInteractiveEvent {
    pub event_id: String,
}

/// Convert an InteractiveEventItem into the data structure for the UI
impl From<&InteractiveEventItem> for InteractiveEventData {
    fn from(item: &InteractiveEventItem) -> Self {
        Self {
            event_id: item.id.clone(),
            title: item.title.clone(),
            description: item.description.clone(),
            faction: item.faction,
            choices: item.choices.clone(),
            popup_urgency: item.popup_urgency,
        }
    }
}
