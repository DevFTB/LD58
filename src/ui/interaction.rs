use crate::factory::physical::remove_physical_link_on_right_click;
use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::prelude::{
    resource_changed, DetectChanges, IntoScheduleConfigs, MouseButton, Plugin, Res, ResMut, Resource,
    Update,
};

pub struct CustomInteractionPlugin;

impl Plugin for CustomInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseButtonEvent::default());
        app.add_systems(
            Update,
            convert_input
                .run_if(resource_changed::<ButtonInput<MouseButton>>)
                .before(remove_physical_link_on_right_click),
        );
    }
}

#[derive(Resource, Default)]
pub struct MouseButtonEvent {
    event: Option<ButtonInput<MouseButton>>,
    is_handled: bool,
}

impl MouseButtonEvent {
    pub(crate) fn handle(&mut self) -> Option<&ButtonInput<MouseButton>> {
        if !self.is_handled {
            self.is_handled = true;
            self.event.as_ref()
        } else {
            None
        }
    }
}

pub fn convert_input(
    button_input: Res<ButtonInput<MouseButton>>,
    mut mbe: ResMut<MouseButtonEvent>,
) {
    if button_input.is_changed() {
        mbe.event = Some(button_input.clone());
        mbe.is_handled = false;
    }
}
