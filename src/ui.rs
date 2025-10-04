// use bevy::{
//     app::{Plugin, Startup, Update},
//     camera::{Camera, Camera2d, Projection},
//     ecs::{
//         query::With,
//         resource::Resource,
//         system::{Commands, Res, Single},
//     },
//     input::{
//         ButtonInput,
//         mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
//     },
//     transform::components::Transform,
// };
use bevy::{
    color::palettes::css::ANTIQUE_WHITE,
    color::palettes::css::GRAY,
    color::palettes::css::BROWN,
    prelude::*
};

pub struct UIPlugin;

const BUILDING_BAR_WIDTH_PCT: f32 = 70.0;
const BUILDING_BAR_HEIGHT_PCT: f32 = 12.0;
const BUILDING_SLOTS: i64 = 10;
const BUILDING_TILE_SIZE: i64 = 64;

const RIGHT_BAR_WIDTH_PCT: f32 = 20.0;

// struct BuildingType{

// }


impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    // spawn the bottom bar with factory draggables
    commands.spawn((
        Node {
            width: percent(BUILDING_BAR_WIDTH_PCT),
            height: percent(BUILDING_BAR_HEIGHT_PCT),
            display: Display::Flex,
            position_type: PositionType::Absolute,
            top: percent(100.0 - BUILDING_BAR_HEIGHT_PCT),
            left: percent((100.0 - BUILDING_BAR_WIDTH_PCT)/2.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(ANTIQUE_WHITE.into()),
    )).with_children(|parent|{
        for _ in 0..BUILDING_SLOTS {
            parent.spawn((
                Node {
                    width: px(BUILDING_TILE_SIZE),
                    height: px(BUILDING_TILE_SIZE),
                    ..default()
                },
                BackgroundColor(GRAY.into())
            ));
        }
    });

    // spawn the right bar with other information: contracts + newsfeed atm
    commands.spawn((
        Node {
            width: percent(RIGHT_BAR_WIDTH_PCT),
            height: percent(100),
            display: Display::Flex,
            position_type: PositionType::Absolute,
            top: percent(0),
            left: percent(100.0 - RIGHT_BAR_WIDTH_PCT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceAround,
            // align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BROWN.into()),
    ));
}