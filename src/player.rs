use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use crate::contracts::{AssociatedWithSink, ContractFulfillment, ContractStatus};
use std::time::Duration;
use crate::factory::logical::DataSink;
use crate::factory::buildings::Tile;
use bevy::platform::collections::HashMap;
use crate::factory::logical::Dataset;
use crate::factory::buildings::sink::ThroughputTracker;

/// Player game state
#[derive(Resource, Debug)]
pub struct Player {
    pub money: i32,
    pub current_year: u32,
    pub net_income: i32,
    // Bankruptcy system
    pub bankruptcy_stage: u32,
    pub bankruptcy_timer: f32, // seconds spent bankrupt in current stage
}

impl Default for Player {
    fn default() -> Self {
        Self {
            money: 1000,
            current_year: 0,
            net_income: 10,
            bankruptcy_stage: 0,
            bankruptcy_timer: 0.0,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Player>()
            .add_systems(Update, (
                update_contract_fulfillment,
                update_money,
            ).chain().run_if(on_timer(Duration::from_secs(1))));
    }
}

/// System that runs every 1 second to update contract fulfillment status
/// TODO: smooth out the throughput calculation over time if necessary
fn update_contract_fulfillment(
    mut contract_query: Query<(&mut ContractFulfillment, &mut Dataset, &AssociatedWithSink, &mut ContractStatus)>,
    mut sink_tile_query: Query<(&mut DataSink, &mut ThroughputTracker, &Tile)>,
) {
    // calculate the throughput per (SinkBuilding entity, dataset) pair
    let mut dataset_sink_throughputs: HashMap<(Entity, Dataset), f32> = HashMap::new();
    for (mut sink, mut throughput_tracker, tile) in sink_tile_query.iter_mut() {
        let sink_building_entity = tile.0;
        if let Some(dataset) = &sink.buffer.shape {
            *dataset_sink_throughputs
                .entry((sink_building_entity, dataset.clone()))
                .or_insert(0.)
                += sink.buffer.last_in;
        }
    }

    // update each contract's fulfillment based on the calculated throughputs
    for (mut fulfillment, dataset, associated_sink, status) in contract_query.iter_mut() {
        if *status != ContractStatus::Active{
            continue; // Only update active contracts
        }
        let sink_building_entity = associated_sink.0;
        if let Some(throughput) = dataset_sink_throughputs.get(&(sink_building_entity, dataset.clone())) {
            fulfillment.update_throughput(*throughput as f64);
        } else {
            fulfillment.update_throughput(0.0);
        }
    }

    // println!("Dataset throughputs: {:?}", dataset_sink_throughputs);


}

// System that runs every 1 second to update player money based on active contracts
fn update_money(
    mut player: ResMut<Player>,
    contract_query: Query<(&ContractStatus, &ContractFulfillment)>,
) {
    let mut total_income = 0.0;
    
    // Calculate income from all active contracts
    for (status, fulfillment) in contract_query.iter() {
        if *status == ContractStatus::Active {
            total_income += fulfillment.get_income();
        }
    }

    // TODO: subtract factory upkeep from total_income
    

    // Update player money and net income
    player.money += (total_income as i32).max(0);
    player.net_income = total_income as i32;

    if player.money == 0 && player.net_income < 0 {
        player.bankruptcy_timer += 1.0;
    }
    
    info!("Player money updated: {} (income: {})", player.money, total_income);
}