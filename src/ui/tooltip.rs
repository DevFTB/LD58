use crate::assets::GameAssets;
use crate::factory::buildings::TileThroughputData;
use crate::factory::logical::calculate_throughput;
use crate::LinkedSpawn;
use bevy::app::{App, Plugin, Update};
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::picking::Pickable;
use bevy::prelude::{
    default, Commands, Component, Deref, DetectChanges, Entity, GlobalTransform, IntoScheduleConfigs,
    On, Out, Over, Pointer, Query, Ref, TextFont, Transform, Visibility,
};
use bevy::sprite::Text2d;
use bevy::text::TextColor;

#[derive(Component, Deref)]
pub struct ToggleOnHover(pub Vec<Entity>);
#[derive(Component)]
pub struct TileThroughputTooltip {
    pub(crate) in_text: Entity,
    pub(crate) out_text: Entity,
}
pub struct TooltipPlugin;
impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(
            move |trigger: On<Pointer<Over>>, mut cmds: Commands, query: Query<&ToggleOnHover>| {
                if let Ok(items) = query.get(trigger.entity) {
                    items.iter().for_each(|i| {
                        cmds.entity(*i).insert(Visibility::Visible);
                    })
                }
            },
        );
        app.add_observer(
            move |trigger: On<Pointer<Out>>, mut cmds: Commands, data: Query<&ToggleOnHover>| {
                if let Some(items) = data.get(trigger.entity).ok() {
                    items.iter().for_each(|c| {
                        cmds.entity(*c).insert(Visibility::Hidden);
                    });
                };
            },
        );

        app.add_systems(Update, update_tooltip.after(calculate_throughput));
    }
}

pub fn update_tooltip(
    mut commands: Commands,
    tooltips: Query<(&TileThroughputTooltip, Ref<TileThroughputData>)>,
) {
    for (tooltip, data) in tooltips {
        if data.is_changed() {
            commands
                .entity(tooltip.in_text)
                .insert(Text2d(data.amount_in.round().to_string()));
            commands
                .entity(tooltip.out_text)
                .insert(Text2d(data.amount_out.round().to_string()));
        }
    }
}

pub fn attach_tooltip(commands: &mut Commands, id: Entity) {
    // Use a deferred command that will access GameFont from the World when executed
    let entity_id = id;
    commands.queue(move |world: &mut bevy::prelude::World| {
        let text_font = {
            let game_assets = world.resource::<GameAssets>();
            game_assets.text_font(40.)
        };
        
        let in_text = world
            .spawn((
                Visibility::Hidden,
                Transform::from_translation(Vec3::new(-64., 0., 0.)),
                Text2d::default(),
                text_font.clone(),
                TextColor(Color::linear_rgba(0., 1.0, 0., 1.0)),
            ))
            .id();
        let out_text = world
            .spawn((
                Visibility::Hidden,
                Transform::from_translation(Vec3::new(64., 0., 0.)),
                Text2d::default(),
                text_font,
                TextColor(Color::linear_rgba(1.0, 0.0, 0., 1.0)),
            ))
            .id();

        let child = world
            .spawn(InheritTranslation(entity_id))
            .add_children(&[in_text, out_text])
            .id();
        world.entity_mut(entity_id).insert((
            TileThroughputData::default(),
            Pickable::default(),
            ToggleOnHover(vec![in_text, out_text]),
            TileThroughputTooltip { in_text, out_text },
            LinkedSpawn(vec![child]),
        ));
    });
}

#[derive(Component, Deref)]
#[require(Transform)]
pub struct InheritTranslation(Entity);

pub fn inherit_translation(
    query: Query<(&InheritTranslation, &mut Transform)>,
    global_transforms: Query<&GlobalTransform>,
) {
    for (inherit, mut transform) in query {
        if let Ok(global_transform) = global_transforms.get(**inherit) {
            transform.translation = global_transform.translation();
        }
    }
}
