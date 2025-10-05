use std::ops::Range;

use bevy::{
    app::{Plugin, Startup, Update},
    camera::{Camera, Camera2d, Projection},
    ecs::{
        query::With,
        resource::Resource,
        system::{Commands, Res, Single},
    },
    input::{
        ButtonInput,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    },
    transform::components::Transform,
};

#[derive(Debug, Resource)]
struct CameraSettings {
    /// Clamp the orthographic camera's scale to this range
    pub orthographic_zoom_range: Range<f32>,
    /// Multiply mouse wheel inputs by this factor when using the orthographic camera
    pub orthographic_zoom_speed: f32,
}

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(CameraSettings {
            // In orthographic projections, we specify camera scale relative to a default value of 1,
            // in which one unit in world space corresponds to one pixel.
            orthographic_zoom_range: 0.5..10.0,
            // This value was hand-tuned to ensure that zooming in and out feels smooth but not slow.
            orthographic_zoom_speed: 0.2,
        });
        app.add_systems(Startup, startup);
        app.add_systems(Update, (zoom, pan_camera));
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn zoom(
    camera: Single<&mut Projection, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    if let Projection::Orthographic(ref mut orthographic) = *camera.into_inner() {
        // We want scrolling up to zoom in, decreasing the scale, so we negate the delta.
        let delta_zoom = -mouse_wheel_input.delta.y * camera_settings.orthographic_zoom_speed;
        // When changing scales, logarithmic changes are more intuitive.
        // To get this effect, we add 1 to the delta, so that a delta of 0
        // results in no multiplicative effect, positive values result in a multiplicative increase,
        // and negative values result in multiplicative decreases.
        let multiplicative_zoom = 1. + delta_zoom;

        orthographic.scale = (orthographic.scale * multiplicative_zoom).clamp(
            camera_settings.orthographic_zoom_range.start,
            camera_settings.orthographic_zoom_range.end,
        );
    }
}

fn pan_camera(
    camera_query: Single<(&mut Transform, &Projection), With<Camera>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    // Only pan when middle mouse button is pressed
    if !mouse_button_input.pressed(MouseButton::Middle) {
        return;
    }

    let (mut camera_transform, projection) = camera_query.into_inner();

    // Get the camera scale to adjust panning speed based on zoom level
    let camera_scale = if let Projection::Orthographic(orthographic) = projection {
        orthographic.scale
    } else {
        1.0
    };

    // Pan the camera based on mouse movement
    // Negate the delta so dragging feels natural (drag right -> camera moves right)
    let delta = -mouse_motion.delta * camera_scale;
    camera_transform.translation.x += delta.x;
    camera_transform.translation.y -= delta.y; // Y is inverted in screen space
}
