/// System to update bankruptcy state and trigger bankruptcy events
pub fn bankruptcy_update_system(
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut event_writer: MessageWriter<ShowInteractiveEvent>,
    library: Res<InteractiveEventLibrary>,
    factions: Res<FactionReputations>,
    event_state: Res<EventState>,
) {
    // Only tick timer if player is bankrupt
    if player.money <= 0 && player.net_income < 0 {
        player.bankruptcy_timer += time.delta().as_secs_f32();
        // Clamp money to 0
        player.money = 0;
        // If timer exceeds threshold, advance stage and trigger event
        let stage_duration = 30.0; // seconds per stage
        if player.bankruptcy_timer >= stage_duration {
            player.bankruptcy_stage += 1;
            player.bankruptcy_timer = 0.0;
            // Find best bankruptcy event for this stage
            let context = GameContext {
                player: &player,
                factions: &factions,
                event_state: &event_state,
            };
            // Event id convention: "bankruptcy_stage_{n}" or similar
            let stage_id = format!("bankruptcy_stage_{}", player.bankruptcy_stage);
            // Find all eligible manual events for this stage
            let mut candidates: Vec<_> = library.events.iter()
                .filter(|e| e.id == stage_id && e.trigger_mode == EventTriggerMode::Manual && context.check_requirements(&e.requirements, e.repeatable, &e.id))
                .collect();
            // Pick the highest-priority event (largest value)
            candidates.sort_by_key(|e| -(e.priority));
            if let Some(event) = candidates.first() {
                let event_data: InteractiveEventData = (*event).into();
                event_writer.write(ShowInteractiveEvent(event_data));
            }
        }
    } else {
        // Not bankrupt, reset timer and stage
        player.bankruptcy_timer = 0.0;
        player.bankruptcy_stage = 0;
    }
}
use bevy::prelude::*;
use rand::Rng;

use super::interactive_events::*;
use crate::factions::FactionReputations;
use crate::player::Player;

/// Timer resource for random event triggering
#[derive(Resource)]
pub struct RandomEventTimer {
    pub timer: Timer,
}

impl Default for RandomEventTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(30.0, TimerMode::Repeating),
        }
    }
}

/// System that handles manual event triggers from game systems
/// This allows any game system to trigger events via TriggerInteractiveEvent message
pub fn handle_manual_event_triggers(
    mut trigger_events: MessageReader<TriggerInteractiveEvent>,
    library: Res<InteractiveEventLibrary>,
    player: Res<Player>,
    factions: Res<FactionReputations>,
    event_state: Res<EventState>,
    mut show_event: MessageWriter<ShowInteractiveEvent>,
) {
    for trigger in trigger_events.read() {
        // Build game context
        let context = GameContext {
            player: &player,
            factions: &factions,
            event_state: &event_state,
        };

        // Check if event exists and can be triggered
        if library.can_trigger_manual_event(&trigger.event_id, &context) {
            if let Some(event) = library.get_event_by_id(&trigger.event_id) {
                let event_data: InteractiveEventData = event.into();
                show_event.write(ShowInteractiveEvent(event_data));
                info!("Manually triggered event: {}", trigger.event_id);
            }
        } else {
            warn!(
                "Cannot trigger event '{}': event not found or requirements not met",
                trigger.event_id
            );
        }
    }
}

/// System that triggers random events periodically
pub fn random_event_trigger_system(
    time: Res<Time>,
    mut timer: ResMut<RandomEventTimer>,
    library: Res<InteractiveEventLibrary>,
    player: Res<Player>,
    factions: Res<FactionReputations>,
    event_state: Res<EventState>,
    queued_events: Res<crate::ui::interactive_event::QueuedEvents>,
    mut event_writer: MessageWriter<ShowInteractiveEvent>,
) {
    if timer.timer.tick(time.delta()).just_finished() {
        // Build game context
        let context = GameContext {
            player: &player,
            factions: &factions,
            event_state: &event_state,
        };

        // Get queued event IDs to filter them out
        let queued_ids: Vec<String> = queued_events.events.iter()
            .map(|e| e.event_id.clone())
            .collect();

        // Get all eligible random events with their weights
        let eligible = library.get_eligible_random_events(&context, time.elapsed_secs_f64(), &queued_ids);
        
        if eligible.is_empty() {
            return;
        }

        // Weighted random selection
        let total_weight: f32 = eligible.iter().map(|(_, weight)| weight).sum();
        let mut rng = rand::rng();
        let mut random = rng.random::<f32>() * total_weight;

        for (idx, weight) in eligible {
            random -= weight;
            if random <= 0.0 {
                let event = &library.events[idx];
                let event_data: InteractiveEventData = event.into();
                event_writer.write(ShowInteractiveEvent(event_data));
                return;
            }
        }
    }
}

/// System that checks for forced events that should auto-trigger
pub fn forced_event_checker_system(
    library: Res<InteractiveEventLibrary>,
    player: Res<Player>,
    factions: Res<FactionReputations>,
    event_state: Res<EventState>,
    queued_events: Res<crate::ui::interactive_event::QueuedEvents>,
    existing_modals: Query<(), With<crate::ui::interactive_event::InteractiveEventModal>>,
    mut event_writer: MessageWriter<ShowInteractiveEvent>,
) {
    // Don't trigger if there's already a modal open or events in queue
    if !existing_modals.is_empty() || !queued_events.events.is_empty() {
        return;
    }

    // Build game context
    let context = GameContext {
        player: &player,
        factions: &factions,
        event_state: &event_state,
    };

    // Get all forced events that should trigger
    let triggered = library.get_triggered_forced_events(&context);

    // Trigger the first forced event (we can only show one at a time)
    if let Some(&idx) = triggered.first() {
        let event = &library.events[idx];
        let event_data: InteractiveEventData = event.into();
        event_writer.write(ShowInteractiveEvent(event_data));
    }
}

/// System that handles player choice consequences
pub fn handle_player_choice_system(
    time: Res<Time>,
    mut choice_events: MessageReader<PlayerChoiceEvent>,
    library: Res<InteractiveEventLibrary>,
    mut player: ResMut<Player>,
    mut factions: ResMut<FactionReputations>,
    mut event_state: ResMut<EventState>,
) {
    for choice_event in choice_events.read() {
        // Find the event by ID
        let event = library.events.iter().find(|e| e.id == choice_event.event_id);
        
        if let Some(event) = event {
            // Mark event as completed
            event_state.complete_event(event.id.clone(), time.elapsed_secs_f64());

            // Get the chosen option
            if let Some(choice) = event.choices.get(choice_event.choice_index) {
                // Apply all consequences
                for consequence in &choice.consequences {
                    match consequence {
                        ConsequenceType::UnlockEvent(event_id) => {
                            event_state.unlock_event(event_id.clone());
                            info!("Unlocked event: {}", event_id);
                        }
                        ConsequenceType::ModifyMoney(amount) => {
                            player.money += amount;
                            info!("Money changed by: {}, new balance: {}", amount, player.money);
                        }
                        ConsequenceType::ModifyReputation { faction, amount } => {
                            factions.add(*faction, *amount);
                            info!("Reputation with {:?} changed by: {}", faction, amount);
                        }
                        ConsequenceType::CompleteEvent(event_id) => {
                            event_state.complete_event(event_id.clone(), time.elapsed_secs_f64());
                            info!("Marked event {} as completed", event_id);
                        }
                        ConsequenceType::Bankruptcy => {
                            player.money = 0;
                            warn!("Player went bankrupt!");
                        }
                        ConsequenceType::UnlockContract(contract_id) => {
                            //TODO: implement contract unlocking
                        }
                    }
                }
            }
        } else {
            warn!("Could not find event with ID: {}", choice_event.event_id);
        }
    }
}
