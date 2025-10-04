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
    color::palettes::css::{ANTIQUE_WHITE, BROWN, GRAY}, ecs::error::info, prelude::*
};

pub struct UIPlugin;

const BUILDING_BAR_WIDTH_PCT: f32 = 70.0;
const BUILDING_BAR_HEIGHT_PCT: f32 = 12.0;
const BUILDING_TILE_SIZE: i64 = 64;

const RIGHT_BAR_WIDTH_PCT: f32 = 20.0;

#[derive(Component, Clone)]
pub struct UIBuilding{
    building_name: String,
    sprite_path: String,
    // size: 
}


impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // i tried really hard to abstract this into a .ron file for way too long but failed horribly. hence what is currently here
    let buildings = [
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
        UIBuilding{building_name: String::from("placeholder"), sprite_path: String::from(r"buildings\building_placeholder.png")},
    ];

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
        for building in &buildings {
            let mut image_node = ImageNode::new(asset_server.load(&building.sprite_path));
            image_node.image_mode = NodeImageMode::Stretch;

            parent.spawn((
                Node {
                    width: px(BUILDING_TILE_SIZE),
                    height: px(BUILDING_TILE_SIZE),
                    ..default()
                },
                image_node,
                // BackgroundColor(GRAY.into()),
                building.clone()
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
        ZIndex(-1),
    ));
}