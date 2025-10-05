use bevy::prelude::*;
use serde::Deserialize;

/// Enum for the four factions in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[repr(u8)]
pub enum Faction {
    Criminal = 0,
    Corporate = 1,
    Government = 2,
    Academia  = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[repr(u8)]
pub enum ReputationLevel {
    Hostile = 0,
    Untrusted = 1,
    Neutral = 2,
    Friendly = 3,
    Trusted = 4,
    Exclusive = 5,
}

/// Placeholder resource for faction reputations.
/// Values range from 0 to 100, starting at 40 (Neutral).
#[derive(Resource, Debug)]
pub struct FactionReputations {
    pub corporate: i32,
    pub academia: i32,
    pub government: i32,
    pub criminal: i32,
}

impl Default for FactionReputations {
    fn default() -> Self {
        Self {
            corporate: 40,
            academia: 40,
            government: 40,
            criminal: 40,
        }
    }
}

impl FactionReputations {
    pub fn get(&self, faction: Faction) -> i32 {
        match faction {
            Faction::Corporate => self.corporate,
            Faction::Academia => self.academia,
            Faction::Government => self.government,
            Faction::Criminal => self.criminal,
        }
    }

    pub fn set(&mut self, faction: Faction, value: i32) {
        let clamped = value.clamp(0, 100);
        match faction {
            Faction::Corporate => self.corporate = clamped,
            Faction::Academia => self.academia = clamped,
            Faction::Government => self.government = clamped,
            Faction::Criminal => self.criminal = clamped,
        }
    }

    pub fn add(&mut self, faction: Faction, delta: i32) {
        let current = self.get(faction);
        self.set(faction, current + delta);
    }
}

/// Plugin for reputation system.
pub struct FactionsPlugin;

impl Plugin for FactionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionReputations>();
    }
}

pub fn reputation_score_to_level(score: u32) -> ReputationLevel {
    match score {
        0..=15 => ReputationLevel::Hostile,   // Hostile
        16..=30 => ReputationLevel::Untrusted,  // Untrusted
        31..=45 => ReputationLevel::Neutral,  // Neutral
        46..=60 => ReputationLevel::Friendly,  // Friendly
        61..=80 => ReputationLevel::Trusted,  // Trusted
        81..=100 => ReputationLevel::Exclusive, // Exclusive
        _ => ReputationLevel::Neutral, // Default to neutral if out of bounds
    }
}