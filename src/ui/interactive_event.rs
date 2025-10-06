use crate::events::interactive_events::{EventChoice, InteractiveEventData, PlayerChoiceEvent, ShowInteractiveEvent};
use crate::factions::Faction;
use bevy::prelude::*;

/// Resource to track when modals were spawned to prevent immediate closure
#[derive(Resource)]
pub struct ModalSpawnCooldown {
    pub timer: Timer,
}

impl Default for ModalSpawnCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct InteractiveEventModal;

/// Component to mark buttons for event choices
#[derive(Component)]
pub struct EventChoiceButton {
    pub choice_index: usize,
}

/// Spawn a choice button
fn spawn_choice_button(commands: &mut Commands, choice: &EventChoice, index: usize) -> Entity {
    let button = commands
        .spawn((
            Button,
            Node {
                padding: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
            BorderRadius::all(Val::Px(4.0)),
            EventChoiceButton {
                choice_index: index,
            },
        ))
        .id();

    let text = commands
        .spawn((
            Text::new(&choice.description),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ))
        .id();

    commands.entity(button).add_children(&[text]);
    button
}

/// System to handle choice button interactions
pub fn handle_choice_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &EventChoiceButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, _button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.45, 0.15));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
        }
    }
}

/// Component to store the event data in the modal for later retrieval
#[derive(Component, Clone)]
pub struct StoredEventData {
    pub event_data: InteractiveEventData,
}

/// Improved system to show the interactive event modal with stored data
pub fn show_interactive_event_system(
    mut commands: Commands,
    mut events: MessageReader<ShowInteractiveEvent>,
    existing_modals: Query<Entity, With<InteractiveEventModal>>,
    mut cooldown: ResMut<ModalSpawnCooldown>,
) {
    // Get the first event (if any)
    if let Some(event) = events.read().next() {
        // Only show one modal at a time - dismiss any existing ones ONLY when showing a new one
        for entity in existing_modals.iter() {
            commands.entity(entity).despawn();
        }

        // Reset the cooldown timer whenever a new modal is spawned
        cooldown.timer.reset();
        spawn_event_modal(&mut commands, event.event_data.clone());
    }
}

/// Spawn the event modal UI with stored data
fn spawn_event_modal(commands: &mut Commands, event_data: InteractiveEventData) {
    let faction_color = match event_data.faction {
        Faction::Academia => Color::srgb(0.2, 0.8, 1.0), // Cyan
        Faction::Corporate => Color::srgb(0.9, 0.9, 0.3), // Yellow
        Faction::Government => Color::srgb(0.3, 1.0, 0.3), // Green
        Faction::Criminal => Color::srgb(1.0, 0.3, 0.3), // Red
    };

    // Create a semi-transparent overlay
    let modal_root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ZIndex(1000),
            InteractiveEventModal,
            StoredEventData {
                event_data: event_data.clone(),
            },
        ))
        .id();

    // Create the modal container
    let modal_container = commands
        .spawn((
            Node {
                width: Val::Px(600.0),
                max_width: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor::all(faction_color),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Text::new(&event_data.title),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(faction_color),
        ))
        .id();

    // Description
    let description = commands
        .spawn((
            Text::new(&event_data.description),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    // Choices container
    let choices_container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .id();

    // Add choice buttons
    let mut choice_buttons = Vec::new();
    for (index, choice) in event_data.choices.iter().enumerate() {
        let button = spawn_choice_button(commands, choice, index);
        choice_buttons.push(button);
    }

    // Build the hierarchy
    commands.entity(modal_root).add_children(&[modal_container]);
    commands
        .entity(modal_container)
        .add_children(&[title, description, choices_container]);
    commands
        .entity(choices_container)
        .add_children(&choice_buttons);
}

/// Improved system to handle choice clicks with stored event data
pub fn handle_choice_click(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &EventChoiceButton), Changed<Interaction>>,
    modal_query: Query<(Entity, &StoredEventData), With<InteractiveEventModal>>,
    mut choice_events: MessageWriter<PlayerChoiceEvent>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cooldown: ResMut<ModalSpawnCooldown>,
    time: Res<Time>,
) {
    // Update the cooldown timer
    cooldown.timer.tick(time.delta());

    // Don't process clicks during cooldown period
    if !cooldown.timer.is_finished() {
        return;
    }

    // Only process if mouse was actually clicked this frame
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Get the stored event data and send the choice
            if let Some((modal_entity, stored_data)) = modal_query.iter().next() {
                if let Some(choice) = stored_data.event_data.choices.get(button.choice_index) {
                    choice_events.write(PlayerChoiceEvent {
                        choice_data: choice.clone(),
                    });

                    info!(
                        "Choice {} selected: {}",
                        button.choice_index, choice.description
                    );
                }

                // Close the modal
                commands.entity(modal_entity).despawn();
            }

            break;
        }
    }
}

pub fn test_trigger_random_event(
    keyboard: Res<ButtonInput<KeyCode>>,
    event_library: Res<crate::events::InteractiveEventLibrary>,
    mut show_event: MessageWriter<ShowInteractiveEvent>,
) {
    use rand::prelude::*;

    if keyboard.just_pressed(KeyCode::KeyE) {
        // Get all available factions and reputation levels
        let factions = [
            Faction::Criminal,
            Faction::Corporate,
            Faction::Government,
            Faction::Academia,
        ];
        let rep_levels = [
            crate::factions::ReputationLevel::Hostile,
            crate::factions::ReputationLevel::Untrusted,
            crate::factions::ReputationLevel::Neutral,
            crate::factions::ReputationLevel::Friendly,
            crate::factions::ReputationLevel::Trusted,
        ];

        let mut rng = rand::rng();

        // Try to find a random event (with retries if the combination doesn't exist)
        for _ in 0..20 {
            let faction = *factions.choose(&mut rng).unwrap();
            let rep_level = *rep_levels.choose(&mut rng).unwrap();

            // Try to get an event for this faction/reputation combo
            if let Some(faction_data) = event_library.0.get(&faction)
                && let Some(events) = faction_data.get(&rep_level)
                && let Some(event_item) = events.choose(&mut rng)
            {
                // Found an event! Create and show it
                let event_data = InteractiveEventData {
                    title: event_item.title.clone(),
                    description: event_item.description.clone(),
                    faction,
                    choices: event_item.choices.clone(),
                };

                show_event.write(ShowInteractiveEvent { event_data });
                info!(
                    "Triggered random event: {} (Faction: {:?}, Level: {:?})",
                    event_item.title, faction, rep_level
                );
                return;
            }
        }

        warn!("Could not find any interactive events in the library!");
    }
}
