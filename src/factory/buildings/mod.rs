use bevy::prelude::{Bundle, Component, Deref, DerefMut};
use bevy::prelude::{Entity, SpawnRelated};
pub mod aggregator;
pub mod buildings;
pub(crate) mod combiner;
pub mod delinker;
pub(crate) mod sink;
pub(crate) mod source;
pub(crate) mod splitter;
pub(crate) mod trunker;

#[derive(Component, Debug, Deref, DerefMut)]
#[relationship_target(relationship = Tile, linked_spawn)]
pub struct Tiles(Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = Tiles)]
pub struct Tile(Entity);
