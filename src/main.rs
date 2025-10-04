use bevy::{math::I8Vec2, prelude::*};

use crate::{
    camera::GameCameraPlugin,
    grid::{Grid, GridPlugin, GridPosition},
};

mod camera;
mod grid;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(GameCameraPlugin)
        .add_plugins(GridPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, grid: Res<Grid>) {
    for i in -5..5 {
        for j in -5..5 {
            commands.spawn((
                GridPosition(I8Vec2 { x: i, y: j }),
                Sprite {
                    color: Color::LinearRgba(LinearRgba {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.5,
                        alpha: 1.0,
                    }),
                    custom_size: Some(Vec2 {
                        x: grid.scale - 8.,
                        y: grid.scale - 8.,
                    }),
                    ..Default::default()
                },
            ));
        }
    }
}
