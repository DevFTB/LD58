use bevy::prelude::*;
use crate::factory::buildings::sink::SinkBuilding;
use serde::{Deserialize, Serialize};

/// Enum for the four factions in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Component)]
#[repr(u8)]
pub enum Faction {
    Criminal = 0,
    #[default]
    Corporate = 1,
    Government = 2,
    Academia = 3,
}

// note: ordering from partialord/ord is independent of the enum discriminant values
// ordering based upon the positon of the enum values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Component, PartialOrd, Ord)]
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

    pub fn get_level(&self, faction: Faction) -> ReputationLevel {
        let rep_val = match faction {
            Faction::Corporate => self.corporate,
            Faction::Academia => self.academia,
            Faction::Government => self.government,
            Faction::Criminal => self.criminal,
        };
        return reputation_score_to_level(rep_val as u32);
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
        app.init_resource::<FactionReputations>()
            .add_systems(Update, lock_unlock_by_reputation_system.run_if(resource_changed::<FactionReputations>));
            // .add_systems(Update, debug_print_locked_unlocked_sinks);
    }
}
/// Debug system to print all locked and unlocked SinkBuilding entities
pub fn debug_print_locked_unlocked_sinks(
    q_locked: Query<Entity, (With<SinkBuilding>, With<Locked>, Without<Unlocked>)>,
    q_unlocked: Query<Entity, (With<SinkBuilding>, With<Unlocked>, Without<Locked>)>,
    mut reputations: ResMut<FactionReputations>,
) {
    let locked: Vec<_> = q_locked.iter().collect();
    let unlocked: Vec<_> = q_unlocked.iter().collect();
    println!("Locked SinkBuildings: {locked:?}");
    println!("Unlocked SinkBuildings: {unlocked:?}");

    // reputations.add(Faction::Corporate, 1);
}

pub fn reputation_score_to_level(score: u32) -> ReputationLevel {
    match score {
        0..=15 => ReputationLevel::Hostile,     // Hostile
        16..=30 => ReputationLevel::Untrusted,  // Untrusted
        31..=45 => ReputationLevel::Neutral,    // Neutral
        46..=60 => ReputationLevel::Friendly,   // Friendly
        61..=80 => ReputationLevel::Trusted,    // Trusted
        81..=100 => ReputationLevel::Exclusive, // Exclusive
        _ => ReputationLevel::Neutral,          // Default to neutral if out of bounds
    }
}

// Unlock and lock system for factions
#[derive(Component)]
pub struct Locked;

#[derive(Component)]
pub struct Unlocked;

/// System to lock or unlock entities based on their faction reputation level
/// if expensive try only run when faction reputation changes or other optimisation
pub fn lock_unlock_by_reputation_system(
    mut commands: Commands,
    q_locked: Query<(Entity, &Faction, &ReputationLevel), (With<Locked>, Without<Unlocked>)>,
    q_unlocked: Query<(Entity, &Faction, &ReputationLevel), (With<Unlocked>, Without<Locked>)>,
    reputations: Res<FactionReputations>,
) {
    // Lock entities if their current reputation level is too low
    for (entity, faction, &level) in q_unlocked.iter() {
        if level > reputations.get_level(*faction) {
            commands.entity(entity).remove::<Unlocked>().insert((Locked,));
        }
    }
    // Unlock entities if their current reputation level is high enough
    for (entity, faction, &level) in q_locked.iter() {
        if level <= reputations.get_level(*faction) {
            commands.entity(entity).remove::<Locked>().insert((Unlocked,));
        }
    }
}