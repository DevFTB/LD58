use bevy::prelude::*;
use crate::events::{AddNewsfeedItemEvent, NewsLibrary};
use crate::factions::{Faction, FactionReputations};
use crate::events::event_data::{get_news_headline};
use crate::assets::GameAssets;
use rand::seq::SliceRandom;

/// Component to mark the root entity of the newsfeed UI.
#[derive(Component)]
pub struct NewsfeedRoot;

/// Component for individual scrolling newsfeed items.
#[derive(Component)]
pub struct NewsfeedItem;

/// Resource to track recently used news event IDs to avoid repetition.
#[derive(Resource, Default)]
pub struct RecentNewsIds {
    pub ids: Vec<u32>,
    pub max_size: usize,
}

impl RecentNewsIds {
    pub fn new(max_size: usize) -> Self {
        Self {
            ids: Vec::new(),
            max_size,
        }
    }
    
    pub fn add(&mut self, id: u32) {
        self.ids.push(id);
        if self.ids.len() > self.max_size {
            self.ids.remove(0);
        }
    }
    
    pub fn clear(&mut self) {
        self.ids.clear();
    }
}

/// Component for choice buttons in interactive events.
#[derive(Component)]
pub struct ChoiceButton {
    pub choice_data: crate::events::EventChoice,
}

/// System to spawn the newsfeed UI on startup.
pub fn spawn_newsfeed_ui(mut commands: Commands) {
    // Spawn a horizontal bar at the top of the screen
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ZIndex(100),
        NewsfeedRoot,
    ));
}

/// System to handle adding newsfeed items - spawns new entities.
pub fn add_newsfeed_item_system(
    mut commands: Commands,
    mut events: MessageReader<AddNewsfeedItemEvent>,
    container_query: Query<Entity, With<NewsfeedRoot>>,
    item_query: Query<(&Node, &ComputedNode), With<NewsfeedItem>>,
    game_assets: Res<GameAssets>,
    windows: Query<&Window>,
) {
    let Ok(container) = container_query.single() else {
        return;
    };
    
    // Get window width to ensure items start off-screen
    let window_width = windows.single().map(|w| w.width()).unwrap_or(800.0);
    
    // Process only one event per frame to avoid width estimation issues
    // Calculate spawn position by finding the rightmost existing item
    let mut spawn_x = window_width;
    for (node, computed) in item_query.iter() {
        if let Val::Px(left) = node.left {
            let width = computed.size().x;
            let right_edge = left + width;
            spawn_x = spawn_x.max(right_edge);
        }
    }

    if let Some(event) = events.read().next() {
        let faction_color = match event.faction {
            Faction::Academia => Color::srgb(0.2, 0.8, 1.0),    // Cyan, index 0
            Faction::Corporate => Color::srgb(0.9, 0.9, 0.3),     // Yellow, index 1
            Faction::Government => Color::srgb(0.3, 1.0, 0.3),   // Green, index 2
            Faction::Criminal => Color::srgb(1.0, 0.3, 0.3),     // Red
        };
        

        // Create a news item container
        let news_item = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(spawn_x),
                    top: Val::Px(0.0),
                    height: Val::Px(40.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    ..default()
                },
                NewsfeedItem,
            ))
            .id();

        // Add faction icon with fixed size and maintain aspect ratio
        let icon = commands
            .spawn((
                ImageNode::from_atlas_image(
                    game_assets.transparent_icons_texture.clone(),
                    TextureAtlas { layout: game_assets.icons_layout.clone(), index: event.faction as usize },
                ),
                Node {
                    width: Val::Px(32.0),  // Set desired size
                    height: Val::Px(32.0),
                    // Auto mode with fixed dimensions will maintain aspect ratio by default
                    ..default()
                },
                BackgroundColor(faction_color)
            ))
            .id();

        // Add text
        let text = commands
            .spawn((
                Text::new(&event.headline),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(faction_color),
                Node {
                    ..default()
                },
            ))
            .id();

        // Add separator
        let separator = commands
            .spawn((
                Text::new(" | "),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    ..default()
                },
            ))
            .id();

        // Parent everything together
        commands.entity(news_item).add_children(&[icon, text, separator]);
        commands.entity(container).add_child(news_item);
    }
}

/// System to scroll newsfeed items from right to left.
pub fn scroll_newsfeed_items(
    mut commands: Commands,
    mut item_query: Query<(Entity, &mut Node), With<NewsfeedItem>>,
    time: Res<Time>,
) {
    let scroll_speed = 50.0;
    let delta = scroll_speed * time.delta_secs();

    for (entity, mut node) in item_query.iter_mut() {
        if let Val::Px(x) = node.left {
            let new_x = x - delta;
            node.left = Val::Px(new_x);
            
            // Despawn items that have scrolled completely off the left edge
            if new_x < -1000.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System to automatically generate newsfeed items periodically.
pub fn generate_news(
    mut events: MessageWriter<AddNewsfeedItemEvent>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    reputations: Res<FactionReputations>,
    news_library: Res<NewsLibrary>,
    mut recent_ids: ResMut<RecentNewsIds>,
) {
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(1.0, TimerMode::Repeating); // Generate news every 5 seconds
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        let mut rng = rand::thread_rng();
        let factions = vec![Faction::Corporate, Faction::Academia, Faction::Government, Faction::Criminal];
        let faction = *factions.choose(&mut rng).unwrap();
        let rep = reputations.get(faction).clamp(0, 100) as u32;

        // get_news_headline handles the loading check internally
        if let Some((id, headline)) = get_news_headline(faction, rep, &news_library, &mut recent_ids.ids) {
            recent_ids.add(id);
            
            events.write(AddNewsfeedItemEvent {
                faction,
                headline,
            });
        }
    }
}