use bevy::prelude::*;

/// Game state for pause management
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    /// Game is running normally - all systems active
    #[default]
    Running,
    /// Paused by event modal - only modal interaction allowed
    /// Time stops, no building placement, no other UI interaction
    EventModal,
    /// Paused by player (spacebar) - building and UI interaction allowed
    /// Time stops, no automatic events, but building placement and UI work
    ManualPause,
}

impl GameState {
    /// Check if the game is paused in any way
    pub fn is_paused(&self) -> bool {
        !matches!(self, GameState::Running)
    }
}

/// System to handle manual pause toggling
pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            GameState::Running => {
                next_state.set(GameState::ManualPause);
                info!("Game paused (manual)");
            }
            GameState::ManualPause => {
                next_state.set(GameState::Running);
                info!("Game resumed");
            }
            GameState::EventModal => {
                // Can't unpause modal with spacebar
            }
        }
    }
}

/// Plugin for pause functionality
pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .add_systems(Update, handle_pause_input);
    }
}
