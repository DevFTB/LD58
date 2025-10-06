use crate::events::{InteractiveEventData, PlayerChoiceEvent, ShowInteractiveEvent, GameContext, EventState, Requirements};
use crate::assets::GameAssets;
use crate::factions::FactionReputations;
use crate::player::Player;
use bevy::prelude::*;
use std::slice::from_ref;

// Event bubble constants
const BUBBLE_SIZE: f32 = 90.0;
const BUBBLE_SPACING: f32 = 10.0;
const BUBBLE_LEFT_OFFSET: f32 = 20.0;
const BUBBLE_BOTTOM_OFFSET: f32 = 20.0;

/// Helper function to check choice requirements and generate disabled reason
fn check_choice_requirements(requirements: &[Requirements], context: &GameContext) -> (bool, Option<String>) {
    
    for req in requirements {
        match req {
            Requirements::MinMoney(amount) => {
                info!("Checking MinMoney: player has ${}, needs ${}", context.player.money, amount);
                if context.player.money < *amount {
                    return (true, Some(format!("Need ${}", amount)));
                }
            }
            Requirements::FactionReputation { faction, min } => {
                let current = context.factions.get(*faction);
                if current < *min {
                    return (true, Some(format!("Need {:?} reputation {}", faction, min)));
                }
            }
            Requirements::MaxMoney(amount) => {
                if context.player.money > *amount {
                    return (true, Some(format!("Too much money (max ${})", amount)));
                }
            }
            Requirements::AllOf(reqs) => {
                let (disabled, reason) = check_choice_requirements(reqs, context);
                if disabled {
                    return (disabled, reason);
                }
            }
            Requirements::AnyOf(reqs) => {
                // Check if ANY requirement is met
                let all_fail = reqs.iter().all(|r| {
                    let (disabled, _) = check_choice_requirements(from_ref(r), context);
                    disabled
                });
                if all_fail {
                    return (true, Some("Requirements not met".to_string()));
                }
            }
            Requirements::NoneOf(reqs) => {
                // Check if ANY requirement is met (which would fail the NoneOf)
                let any_met = reqs.iter().any(|r| {
                    let (disabled, _) = check_choice_requirements(from_ref(r), context);
                    !disabled
                });
                if any_met {
                    return (true, Some("Requirements conflict".to_string()));
                }
            }
            // Other requirements that may not apply to choices
            _ => {}
        }
    }
    
    (false, None)
}

/// Resource to track when modals were spawned to prevent immediate closure
#[derive(Resource)]
pub struct ModalSpawnCooldown {
    pub timer: Timer,
}

impl Default for ModalSpawnCooldown {
    fn default() -> Self {
        Self {
            // Start with a finished timer so buttons work immediately
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

impl ModalSpawnCooldown {
    /// Mark that a modal was just spawned
    pub fn just_spawned(&mut self) {
        self.timer.reset();
    }
    
    /// Check if enough time has passed since spawn
    pub fn is_ready(&self) -> bool {
        self.timer.is_finished()
    }
}

#[derive(Component)]
pub struct InteractiveEventModal;

/// Component to mark buttons for event choices
#[derive(Component)]
pub struct EventChoiceButton {
    pub choice_index: usize,
    pub is_disabled: bool,
    pub disabled_reason: Option<String>,
}

/// Component to mark text that should scale with window size
#[derive(Component)]
pub struct ScalableText {
    /// Base size as a percentage of window width
    pub base_vw: f32,
}

impl ScalableText {
    pub fn from_vw(vw: f32) -> Self {
        Self { base_vw: vw }
    }
}

/// Spawn a choice button with visual indicators for consequences
fn spawn_choice_button(
    commands: &mut Commands,
    choice: &str,
    index: usize,
    is_disabled: bool,
    disabled_reason: Option<String>,
    consequences: &[crate::events::ConsequenceType],
    game_assets: &crate::assets::GameAssets,
) -> Entity {
    use crate::events::ConsequenceType;
    
    let (bg_color, text_color) = if is_disabled {
        (Color::srgb(0.15, 0.15, 0.15), Color::srgb(0.5, 0.5, 0.5))
    } else {
        (Color::srgb(0.25, 0.25, 0.25), Color::srgb(0.9, 0.9, 0.9))
    };

    let button = commands
        .spawn((
            Button,
            Node {
                padding: UiRect::all(Val::Vh(1.2)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Vw(1.0),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
            BorderRadius::all(Val::Px(4.0)),
            EventChoiceButton {
                choice_index: index,
                is_disabled,
                disabled_reason: disabled_reason.clone(),
            },
        ))
        .id();

    let text = commands
        .spawn((
            Text::new(choice),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(text_color),
            ScalableText::from_vw(1.2),
        ))
        .id();

    // Create container for consequence indicators
    let indicators_container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Vw(0.5),
            ..default()
        })
        .id();

    // Add visual indicators for all consequences
    let mut indicators = Vec::new();
    for consequence in consequences {
        match consequence {
            ConsequenceType::ModifyReputation { faction, amount } => {
                // Create faction icon + arrow indicator
                let indicator = spawn_faction_consequence_indicator(
                    commands,
                    *faction,
                    *amount,
                    game_assets,
                );
                indicators.push(indicator);
            }
            ConsequenceType::ModifyMoney(amount) => {
                // Create money icon + arrow indicator
                let indicator = spawn_money_consequence_indicator(
                    commands,
                    *amount,
                    game_assets,
                );
                indicators.push(indicator);
            }
            ConsequenceType::Bankruptcy => {
                // Warning indicator for bankruptcy
                let indicator = spawn_text_consequence_indicator(
                    commands,
                    "B",
                    Color::srgb(1.0, 0.3, 0.3),
                );
                indicators.push(indicator);
            }
            _ => {
                // Other consequence types can be added here   
            }
        }
    }

    commands.entity(indicators_container).add_children(&indicators);
    commands.entity(button).add_children(&[text, indicators_container]);
    button
}

/// Spawn a visual indicator for faction reputation change
fn spawn_faction_consequence_indicator(
    commands: &mut Commands,
    faction: crate::factions::Faction,
    amount: i32,
    game_assets: &crate::assets::GameAssets,
) -> Entity {
    let container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Vw(0.2),
            ..default()
        })
        .id();

    // Faction icon
    let faction_icon_index = game_assets.faction_icon(faction);
    let faction_icon = commands
        .spawn((
            ImageNode::from_atlas_image(
                game_assets.small_sprites_texture.clone(),
                TextureAtlas {
                    layout: game_assets.small_sprites_layout.clone(),
                    index: faction_icon_index,
                },
            ),
            Node {
                width: Val::Vw(1.5),
                height: Val::Vw(1.5),
                ..default()
            },
        ))
        .id();

    // Arrow indicator based on amount
    let arrow_index = if amount >= 10 {
        game_assets.utility_icons.arrow_double_up
    } else if amount > 0 {
        game_assets.utility_icons.arrow_up
    } else if amount <= -10 {
        game_assets.utility_icons.arrow_double_down
    } else {
        game_assets.utility_icons.arrow_down
    };

    let arrow_icon = commands
        .spawn((
            ImageNode::from_atlas_image(
                game_assets.small_sprites_texture.clone(),
                TextureAtlas {
                    layout: game_assets.small_sprites_layout.clone(),
                    index: arrow_index,
                },
            ),
            Node {
                width: Val::Vw(1.2),
                height: Val::Vw(1.2),
                ..default()
            },
        ))
        .id();

    commands.entity(container).add_children(&[faction_icon, arrow_icon]);
    container
}

/// Spawn a visual indicator for money changes
fn spawn_money_consequence_indicator(
    commands: &mut Commands,
    amount: i32,
    game_assets: &crate::assets::GameAssets,
) -> Entity {
    let container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Vw(0.2),
            ..default()
        })
        .id();

    // Money icon
    let money_icon = commands
        .spawn((
            ImageNode::from_atlas_image(
                game_assets.small_sprites_texture.clone(),
                TextureAtlas {
                    layout: game_assets.small_sprites_layout.clone(),
                    index: game_assets.utility_icons.money,
                },
            ),
            Node {
                width: Val::Vw(1.5),
                height: Val::Vw(1.5),
                ..default()
            },
        ))
        .id();

    // Arrow indicator based on amount
    let arrow_index = if amount >= 1000 {
        game_assets.utility_icons.arrow_double_up
    } else if amount > 0 {
        game_assets.utility_icons.arrow_up
    } else if amount <= -1000 {
        game_assets.utility_icons.arrow_double_down
    } else {
        game_assets.utility_icons.arrow_down
    };

    let arrow_icon = commands
        .spawn((
            ImageNode::from_atlas_image(
                game_assets.small_sprites_texture.clone(),
                TextureAtlas {
                    layout: game_assets.small_sprites_layout.clone(),
                    index: arrow_index,
                },
            ),
            Node {
                width: Val::Vw(1.2),
                height: Val::Vw(1.2),
                ..default()
            },
        ))
        .id();

    commands.entity(container).add_children(&[money_icon, arrow_icon]);
    container
}

/// Spawn a simple text-based consequence indicator
fn spawn_text_consequence_indicator(
    commands: &mut Commands,
    emoji: &str,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Text::new(emoji),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(color),
            ScalableText::from_vw(1.5),
        ))
        .id()
}

/// System to handle choice button interactions
pub fn handle_choice_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &EventChoiceButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        if button.is_disabled {
            // Keep disabled styling
            *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            continue;
        }

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

/// System to scale text based on window size
pub fn scale_text_system(
    windows: Query<&Window>,
    mut text_query: Query<(&ScalableText, &mut TextFont)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    
    let window_width = window.width();
    
    for (scalable, mut font) in text_query.iter_mut() {
        // Calculate font size as percentage of window width
        let new_size = (scalable.base_vw / 100.0) * window_width;
        // Clamp to reasonable values
        let new_size = new_size.clamp(10.0, 100.0);
        
        if (font.font_size - new_size).abs() > 0.5 {
            font.font_size = new_size;
        }
    }
}

/// Component to store the event data in the modal for later retrieval
#[derive(Component, Clone)]
pub struct StoredEventData {
    pub event_data: InteractiveEventData,
}

/// Component to mark a tooltip entity and track which button it belongs to
#[derive(Component)]
pub struct ChoiceTooltip {
    parent_button: Entity,
}

/// System to show/hide tooltips on hover for disabled choices
pub fn handle_choice_tooltip(
    mut commands: Commands,
    button_query: Query<(Entity, &Interaction, &EventChoiceButton)>,
    tooltip_query: Query<(Entity, &ChoiceTooltip)>,
    windows: Query<&Window>,
) {
    // Remove tooltips for buttons that are no longer hovered
    for (tooltip_entity, tooltip) in tooltip_query.iter() {
        if let Ok((_, interaction, _)) = button_query.get(tooltip.parent_button) {
            if *interaction != Interaction::Hovered {
                commands.entity(tooltip_entity).despawn();
            }
        } else {
            // Button no longer exists
            commands.entity(tooltip_entity).despawn();
        }
    }
    
    // Show tooltip when hovering over disabled button
    for (button_entity, interaction, button) in button_query.iter() {
        if *interaction == Interaction::Hovered && button.is_disabled
            && let Some(reason) = &button.disabled_reason {
                // Check if tooltip already exists for this button
                let tooltip_exists = tooltip_query.iter().any(|(_, t)| t.parent_button == button_entity);
                
                if !tooltip_exists {
                    // Get cursor position if available
                    let (cursor_x, cursor_y) = if let Ok(window) = windows.single() {
                        if let Some(cursor_pos) = window.cursor_position() {
                            (cursor_pos.x, cursor_pos.y)
                        } else {
                            (100.0, 100.0)
                        }
                    } else {
                        (100.0, 100.0)
                    };
                    
                    // Spawn tooltip at cursor position (not as a child)
                    commands.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(cursor_x + 15.0), // Slight offset from cursor
                            top: Val::Px(cursor_y + 15.0),
                            padding: UiRect::all(Val::Vw(0.8)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                        BorderColor::all(Color::srgb(0.9, 0.4, 0.4)),
                        BorderRadius::all(Val::Px(4.0)),
                        ZIndex(2000),
                        ChoiceTooltip {
                            parent_button: button_entity,
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(reason),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.8, 0.8)),
                            ScalableText::from_vw(1.1),
                        ));
                    });
                }
            }
    }
}

/// OLD system - replaced by route_events_by_urgency
/// Kept for reference, but no longer used
#[allow(dead_code)]
pub fn show_interactive_event_system_old(
    mut commands: Commands,
    mut events: MessageReader<ShowInteractiveEvent>,
    existing_modals: Query<Entity, With<InteractiveEventModal>>,
    mut cooldown: ResMut<ModalSpawnCooldown>,
    game_assets: Res<GameAssets>,
    player: Res<Player>,
    factions: Res<FactionReputations>,
    event_state: Res<EventState>,
) {
    // Get the first event (if any)
    if let Some(event) = events.read().next() {
        // Only show one modal at a time - dismiss any existing ones ONLY when showing a new one
        for entity in existing_modals.iter() {
            commands.entity(entity).despawn();
        }

        // Mark that a modal was just spawned (prevents immediate re-close)
        cooldown.just_spawned();
        
        // Build game context for requirement checking
        let context = GameContext {
            player: &player,
            factions: &factions,
            event_state: &event_state,
        };
        
        spawn_event_modal(&mut commands, event.0.clone(), &game_assets, &context);
    }
}

/// Spawn the event modal UI with stored data
fn spawn_event_modal(commands: &mut Commands, event_data: InteractiveEventData, game_assets: &GameAssets, context: &GameContext) {
    // Use faction color if available, otherwise use default
    let border_color = event_data.faction
        .map(|f| game_assets.faction_color(f))
        .unwrap_or(Color::srgba(0.2, 0.6, 0.9, 1.0));

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
                width: Val::Vw(65.0),
                max_width: Val::Vw(70.0),
                height: Val::Auto,
                max_height: Val::Vh(85.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Vw(2.5)),
                row_gap: Val::Vh(2.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor::all(border_color),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .id();

    // Create header container with optional faction icon + title
    let header_container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .id();

    // Add faction icon if faction is present
    if let Some(faction) = event_data.faction {
        let icon_index = game_assets.faction_icon(faction);
        let icon = commands
            .spawn((
                ImageNode::from_atlas_image(
                    game_assets.small_sprites_texture.clone(),
                    TextureAtlas { 
                        layout: game_assets.small_sprites_layout.clone(), 
                        index: icon_index
                    },
                ),
                Node {
                    width: Val::Vw(3.5),
                    height: Val::Vw(3.5),
                    ..default()
                },
                BackgroundColor(border_color),
            ))
            .id();
        commands.entity(header_container).add_children(&[icon]);
    }

    // Title
    let title = commands
        .spawn((
            Text::new(&event_data.title),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(border_color),
            ScalableText::from_vw(2.0),
        ))
        .id();

    commands.entity(header_container).add_children(&[title]);

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
                margin: UiRect::bottom(Val::Vh(1.5)),
                ..default()
            },
            ScalableText::from_vw(1.3),
        ))
        .id();

    // Choices container
    let choices_container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Vh(1.5),
            ..default()
        })
        .id();

    // Add choice buttons
    let mut choice_buttons = Vec::new();
    for (index, choice) in event_data.choices.iter().enumerate() {
        // Check if requirements are met
        let (is_disabled, disabled_reason) = check_choice_requirements(&choice.requirements, context);
        
        info!("Choice {} '{}': disabled={}, reason={:?}, requirements={:?}", 
              index, choice.text, is_disabled, disabled_reason, choice.requirements);
        
        let button = spawn_choice_button(
            commands, 
            &choice.text, 
            index, 
            is_disabled, 
            disabled_reason,
            &choice.consequences,
            game_assets,
        );
        choice_buttons.push(button);
    }

    // Build the hierarchy
    commands.entity(modal_root).add_children(&[modal_container]);
    commands
        .entity(modal_container)
        .add_children(&[header_container, description, choices_container]);
    commands
        .entity(choices_container)
        .add_children(&choice_buttons);
}

pub fn handle_choice_click(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &EventChoiceButton), Changed<Interaction>>,
    modal_query: Query<(Entity, &StoredEventData), With<InteractiveEventModal>>,
    mut choice_events: MessageWriter<PlayerChoiceEvent>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Ignore clicks on disabled buttons
            if button.is_disabled {
                if let Some(reason) = &button.disabled_reason {
                    warn!("Cannot select choice: {}", reason);
                }
                continue;
            }

            // Get the stored event data and send the choice
            if let Some((modal_entity, stored_data)) = modal_query.iter().next() {
                choice_events.write(PlayerChoiceEvent {
                    event_id: stored_data.event_data.event_id.clone(),
                    choice_index: button.choice_index,
                });

                info!(
                    "Choice {} selected for event: {}",
                    button.choice_index, stored_data.event_data.event_id
                );

                // Close the modal
                commands.entity(modal_entity).despawn();
            }

            break;
        }
    }
}

pub fn test_trigger_random_event(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    event_library: Res<crate::events::InteractiveEventLibrary>,
    player: Res<crate::player::Player>,
    factions: Res<crate::factions::FactionReputations>,
    event_state: Res<crate::events::EventState>,
    queued_events: Res<QueuedEvents>,
    mut show_event: MessageWriter<ShowInteractiveEvent>,
) {
    use rand::prelude::*;

    if keyboard.just_pressed(KeyCode::KeyE) {
        // Build game context (same as random_event_trigger_system)
        let context = crate::events::GameContext {
            player: &player,
            factions: &factions,
            event_state: &event_state,
        };

        // Get queued event IDs to filter them out
        let queued_ids: Vec<String> = queued_events.events.iter()
            .map(|e| e.event_id.clone())
            .collect();

        // Get all eligible random events with their weights (filters by requirements and cooldown)
        let eligible = event_library.get_eligible_random_events(&context, time.elapsed_secs_f64(), &queued_ids);
        
        if eligible.is_empty() {
            warn!("No eligible random events found!");
            return;
        }

        // Weighted random selection (same logic as random_event_trigger_system)
        let total_weight: f32 = eligible.iter().map(|(_, weight)| weight).sum();
        let mut rng = rand::rng();
        let mut random = rng.random::<f32>() * total_weight;

        for (idx, weight) in eligible {
            random -= weight;
            if random <= 0.0 {
                let event = &event_library.events[idx];
                let event_data: InteractiveEventData = event.into();
                show_event.write(ShowInteractiveEvent(event_data));
                info!("Triggered random event (test): {}", event.title);
                return;
            }
        }
    }
}

// ============== Event Bubble Queue System ==============

/// Resource to store queued non-urgent events
#[derive(Resource, Default, Debug)]
pub struct QueuedEvents {
    pub events: Vec<InteractiveEventData>,
}

/// Component marking an event bubble in the bottom left
#[derive(Component, Debug)]
pub struct EventBubble {
    pub event_data: InteractiveEventData,
}

/// Component for wobble animation
#[derive(Component)]
pub struct BubbleWobble {
    pub timer: f32,
    pub cycle_timer: f32,        // Time until next wobble
    pub cycle_duration: f32,     // How often to wobble (~5 seconds)
    pub wobble_duration: f32,    // How long each wobble lasts
    pub is_wobbling: bool,       // Whether currently wobbling
    pub frequency: f32,
    pub amplitude: f32,
}

/// System that handles event routing based on urgency
pub fn route_events_by_urgency(
    mut show_events: MessageReader<ShowInteractiveEvent>,
    mut queued_events: ResMut<QueuedEvents>,
    mut commands: Commands,
    existing_modals: Query<Entity, With<InteractiveEventModal>>,
    mut cooldown: ResMut<ModalSpawnCooldown>,
    game_assets: Res<GameAssets>,
    player: Res<Player>,
    factions: Res<FactionReputations>,
    event_state: Res<EventState>,
) {
    for event in show_events.read() {
        if event.0.popup_urgency {
            // Urgent event - show immediately
            // Only show one modal at a time - dismiss any existing ones
            for entity in existing_modals.iter() {
                commands.entity(entity).despawn();
            }

            cooldown.just_spawned();
            
            let context = GameContext {
                player: &player,
                factions: &factions,
                event_state: &event_state,
            };
            
            spawn_event_modal(&mut commands, event.0.clone(), &game_assets, &context);
        } else {
            // Non-urgent event - add to queue only if not already queued
            if !queued_events.events.iter().any(|e| e.event_id == event.0.event_id) {
                queued_events.events.push(event.0.clone());
            }
        }
    }
}

/// System that spawns/updates event bubbles in the bottom left
pub fn manage_event_bubbles(
    mut commands: Commands,
    queued_events: Res<QueuedEvents>,
    existing_bubbles: Query<(Entity, &EventBubble)>,
    game_assets: Res<GameAssets>,
) {
    // Check if queued events changed
    if !queued_events.is_changed() {
        return;
    }

    // Despawn all existing bubbles
    for (entity, _) in existing_bubbles.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new bubbles for all queued events
    for (index, event_data) in queued_events.events.iter().enumerate() {
        spawn_event_bubble(&mut commands, event_data.clone(), index, &game_assets);
    }
}

/// Spawn a single event bubble
fn spawn_event_bubble(
    commands: &mut Commands,
    event_data: InteractiveEventData,
    index: usize,
    game_assets: &GameAssets,
) {
    // Calculate position (stack upwards)
    let bottom_position = BUBBLE_BOTTOM_OFFSET + (index as f32) * (BUBBLE_SIZE + BUBBLE_SPACING);
    
    // Get faction color or default
    let bubble_color = event_data.faction
        .map(|f| game_assets.faction_color(f))
        .unwrap_or(Color::srgba(0.2, 0.6, 0.9, 1.0));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(BUBBLE_SIZE),
                height: Val::Px(BUBBLE_SIZE),
                left: Val::Px(BUBBLE_LEFT_OFFSET),
                bottom: Val::Px(bottom_position),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(3.0)),
                overflow: Overflow::clip(), // Clip children to circular shape
                ..default()
            },
            BackgroundColor(bubble_color.with_alpha(0.8)),
            BorderColor::all(bubble_color),
            BorderRadius::all(Val::Px(BUBBLE_SIZE / 2.0)), // Make it circular
            EventBubble {
                event_data: event_data.clone(),
            },
            BubbleWobble {
                timer: 0.0,
                cycle_timer: (index as f32) * 0.5, // Stagger start times
                cycle_duration: 5.0,
                wobble_duration: 1.0,               // Wobble for 1 second
                is_wobbling: false,
                frequency: 3.0 + (index as f32) * 0.3, // Vary frequency per bubble
                amplitude: 4.0,
            },
            Interaction::default(),
        ))
        .with_children(|parent| {
            // Add faction sprite icon if available
            if let Some(faction) = event_data.faction {
                let icon_index = game_assets.faction_icon(faction);
                parent.spawn((
                    ImageNode::from_atlas_image(
                        game_assets.small_sprites_texture.clone(),
                        TextureAtlas {
                            layout: game_assets.small_sprites_layout.clone(),
                            index: icon_index,
                        },
                    ),
                    Node {
                        width: Val::Px(48.0),
                        height: Val::Px(48.0),
                        ..default()
                    },
                ));
            } else {
                // Fallback to text indicator
                parent.spawn((
                    Text::new("!"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            }
        });
}

/// System that handles clicking on event bubbles
pub fn handle_bubble_clicks(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &EventBubble), Changed<Interaction>>,
    mut queued_events: ResMut<QueuedEvents>,
    existing_modals: Query<Entity, With<InteractiveEventModal>>,
    mut cooldown: ResMut<ModalSpawnCooldown>,
    game_assets: Res<GameAssets>,
    player: Res<Player>,
    factions: Res<FactionReputations>,
    event_state: Res<EventState>,
) {
    for (interaction, bubble) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Remove this event from the queue
            queued_events.events.retain(|e| e.event_id != bubble.event_data.event_id);
            
            // Close any existing modals
            for entity in existing_modals.iter() {
                commands.entity(entity).despawn();
            }
            
            cooldown.just_spawned();
            
            let context = GameContext {
                player: &player,
                factions: &factions,
                event_state: &event_state,
            };
            
            // Show the modal
            spawn_event_modal(&mut commands, bubble.event_data.clone(), &game_assets, &context);
        }
    }
}

/// System that animates event bubbles with a wobble effect
pub fn animate_bubble_wobble(
    time: Res<Time>,
    mut bubbles: Query<(&mut BubbleWobble, &mut Node), With<EventBubble>>,
) {
    for (mut wobble, mut node) in bubbles.iter_mut() {
        // Update cycle timer
        wobble.cycle_timer += time.delta_secs();
        
        // Check if it's time to start a new wobble
        if wobble.cycle_timer >= wobble.cycle_duration && !wobble.is_wobbling {
            wobble.is_wobbling = true;
            wobble.timer = 0.0;
            wobble.cycle_timer = 0.0;
        }
        
        // Update wobble timer if actively wobbling
        if wobble.is_wobbling {
            wobble.timer += time.delta_secs();
            
            // Stop wobbling after duration
            if wobble.timer >= wobble.wobble_duration {
                wobble.is_wobbling = false;
                wobble.timer = 0.0;
            }
        }
        
        // Calculate wobble offset using sine wave (only if wobbling)
        let (offset_x, offset_y) = if wobble.is_wobbling {
            // Create a smooth ease-out effect
            let progress = wobble.timer / wobble.wobble_duration;
            let ease = 1.0 - (1.0 - progress).powi(3); // Ease out cubic
            let decay = 1.0 - ease; // Amplitude decays as wobble progresses
            
            let angle = wobble.timer * wobble.frequency * std::f32::consts::TAU;
            let x = angle.sin() * wobble.amplitude * decay;
            let y = (angle * 1.5).cos() * wobble.amplitude * 0.5 * decay;
            (x, y)
        } else {
            (0.0, 0.0)
        };
        
        // Apply wobble to position
        if let Val::Px(_left) = node.left {
            // Base left offset, add wobble
            node.left = Val::Px(BUBBLE_LEFT_OFFSET + offset_x);
        }
        
        if let Val::Px(bottom) = node.bottom {
            // Keep the base bottom position but add wobble
            // Extract the base position (without previous wobble)
            // Calculate which bubble this is based on bottom position
            let index = ((bottom - BUBBLE_BOTTOM_OFFSET) / (BUBBLE_SIZE + BUBBLE_SPACING)).round();
            let base_bottom = BUBBLE_BOTTOM_OFFSET + index * (BUBBLE_SIZE + BUBBLE_SPACING);
            
            node.bottom = Val::Px(base_bottom + offset_y);
        }
    }
}
