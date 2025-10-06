use bevy::ecs::entity;
use bevy::{prelude::*};
use bevy::ecs::relationship::{RelationshipTarget};
use serde::Deserialize;
use crate::factory::logical::{Dataset};
use crate::factions::{Faction, ReputationLevel, Unlocked};
use bevy::platform::collections::HashMap;
use rand::seq::SliceRandom;
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalRng;
use bevy::time::common_conditions::on_timer;
use crate::factory::buildings::sink::{self, SinkBuilding};
use rand::prelude::IndexedRandom;

// Add the Deserialize trait to your existing components that are in the RON file
#[derive(Component, Deserialize, Debug)]
pub struct Contract;

#[derive(Component, Deserialize, Debug)]
pub struct ContractTimeout(pub f32);

#[derive(Component, Deserialize, Debug, PartialEq, Eq)]
pub enum ContractStatus {
    Pending,
    Active,
    Completed,
    Rejected,
    Failed,
}

#[derive(Component, Debug)]
pub struct FailingTimer {
    pub timer: Timer,
}

impl FailingTimer {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ContractFulfillmentStatus {
    Exceeding,
    Meeting,
    Failing,
}


#[derive(Component, Default, Deserialize, Clone, Debug)]
pub struct ContractDescription {
    pub name: String,
    pub description: String,
}

// --- New Structs for RON loading ---

// Represents a single contract definition from the RON file
#[derive(Debug, Deserialize, Clone)]
pub struct ContractDefinition {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub faction: Faction,
    pub reputation: ReputationLevel,
    pub base_threshold: f64,
    pub base_money: f64,
    pub dataset: Dataset,
}

// A resource to hold all contracts loaded from the RON file
#[derive(Resource, Debug, Default)]
pub struct ContractLibrary {
    pub contracts: HashMap<u32, ContractDefinition>,
}

impl ContractLibrary {
    pub fn all_contracts(&self) -> Vec<&ContractDefinition> {
        self.contracts.values().collect()
    }
}

#[derive(Component)]
#[relationship(relationship_target = SinkContracts)]
pub struct AssociatedWithSink(pub Entity);

#[derive(Component, Debug, Default)]
#[relationship_target(relationship = AssociatedWithSink)]
pub struct SinkContracts(Vec<Entity>);

impl SinkContracts {
    pub fn contracts(&self) -> &[Entity] {
        &self.0
    }

    // ai did this not 100% sure it works but gonna trust it
    pub fn get_current_contracts(&self, contract_query: &Query<&ContractStatus>) -> Vec<Entity> {
    self.0.iter()
        .filter(|&&contract_entity| {
            if let Ok(status) = contract_query.get(contract_entity) {
                matches!(status, ContractStatus::Pending | ContractStatus::Active)
            } else {
                false
            }
        })
        .copied()
        .collect()
    }
}


// duplicates base_threshold and base_money in ContractBundle but i think its ok
#[derive(Component, Debug)]
pub struct ContractFulfillment {
    pub throughput: f64,
    pub status: ContractFulfillmentStatus,
    pub base_threshold: f64,
    pub base_money: f64,
}

impl ContractFulfillment {
    /// Calculate the current money per second for this contract, given its base money rate.
    pub fn get_income(&self) -> f64 {
        match self.status {
            ContractFulfillmentStatus::Exceeding => self.base_money * 2.0,
            ContractFulfillmentStatus::Meeting => self.base_money,
            ContractFulfillmentStatus::Failing => 0.,
        }
    }

    pub fn update_throughput(&mut self, new_throughput: f64) {
        self.throughput = new_throughput;
        self.status = self.get_fulfillment_status();
    }

    fn get_fulfillment_status(&mut self) -> ContractFulfillmentStatus {
        let threshold_fraction = self.throughput / self.base_threshold;
        get_fulfillment_status(threshold_fraction)
    }

    pub fn new(base_threshold: f64, base_money: f64) -> Self {
        Self {
            throughput: 0.0,
            status: ContractFulfillmentStatus::Failing,
            base_threshold,
            base_money,
        }
    }

}


// baciscally all contract entities will have an AssociatedWithSink component as well apart from debug ones
#[derive(Bundle, Debug)]
pub struct ContractBundle {
    pub contract: Contract,
    pub status: ContractStatus,
    pub dataset: Dataset,
    pub faction: Faction,
    pub timeout: ContractTimeout,
    pub description: ContractDescription,
    pub fulfillment_info: ContractFulfillment,
}

const MAX_CONTRACTS_PER_SINK: usize = 4;
const MAX_PENDING_CONTRACTS: usize = 3;

// --- Resources ---

#[derive(Resource)]
struct GameTimer {
    timer: Timer,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(60.0, TimerMode::Once), // 1 minute timer
        }
    }
}

// --- Plugin and Systems ---

pub struct ContractsPlugin;

impl Plugin for ContractsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_contracts_from_ron)
            .init_resource::<GameTimer>()
            .add_systems(Update, (
                first_minute_system,
                generate_random_pending_contract_system.run_if(on_timer(std::time::Duration::from_secs(20))),
                handle_failing_contract_timeouts,
            ));
    }
}
// um super sus but not a lot of time left go ai
/// System that runs only during the first 1 minute of the game
fn first_minute_system(
    mut game_timer: ResMut<GameTimer>,
    time: Res<Time>,
    mut commands: Commands,
    contract_library: Res<ContractLibrary>,
    sinks: Query<(Entity, &Faction, &ReputationLevel, &SinkContracts), (With<Unlocked>, With<SinkBuilding>)>,
    contract_query: Query<&ContractStatus>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    if contract_query.iter().filter(|&status| *status == ContractStatus::Pending).count() >= MAX_PENDING_CONTRACTS {
        // Already at max pending contracts
        return;
    }
    // Tick the timer
    game_timer.timer.tick(time.delta());

    // Only run if the first minute hasn't passed yet
    if !game_timer.timer.finished() {
        // Example: Generate contracts more frequently during the first minute
        // This could be any logic you want to run only in the first minute
        
        // For demonstration, let's generate a contract every 5 seconds during the first minute
        if game_timer.timer.elapsed_secs() > 0.0 && game_timer.timer.elapsed_secs() % 5.0 < time.delta_secs() {
            // Only consider sinks that are not full
            let sink_entities: Vec<_> = sinks
                .iter()
                .filter(|(_, _, _, sink_contracts)| {
                    sink_contracts.get_current_contracts(&contract_query).len() < MAX_CONTRACTS_PER_SINK
                })
                .collect();

            if let Some((sink_entity, faction, reputation, _)) = sink_entities.choose(&mut rng) {
                if let Some(contract_bundle) = find_and_generate_contract(**faction, **reputation, &contract_library) {
                    let contract_entity = commands.spawn(contract_bundle).id();
                    commands.entity(contract_entity).insert(AssociatedWithSink(*sink_entity));
                    info!("Generated first-minute contract {:?} for sink {:?} at {:.1}s", 
                          contract_entity, sink_entity, game_timer.timer.elapsed_secs());
                }
            }
        }
    }
    // After 1 minute, this system will stop doing anything but will still run
    // You could also remove the system entirely after the timer finishes if needed
}

/// System to generate a new pending random contract every 2 minutes and link it to a random SinkBuilding
fn generate_random_pending_contract_system(
    mut commands: Commands,
    contract_library: Res<ContractLibrary>,
    sinks: Query<(Entity, &Faction, &ReputationLevel, &SinkContracts), (With<Unlocked>, With<SinkBuilding>)>,
    contract_query: Query<&ContractStatus>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>
) {
    // Only consider sinks that are not full
    let sink_entities: Vec<_> = sinks
        .iter()
        .filter(|(_, _, _, sink_contracts)| {
            sink_contracts.get_current_contracts(&contract_query).len() < MAX_CONTRACTS_PER_SINK
        })
        .collect();

    if contract_query.iter().filter(|&status| *status == ContractStatus::Pending).count() >= MAX_PENDING_CONTRACTS {
        // Already at max pending contracts
        return;
    }

    if let Some((sink_entity, faction, reputation, _)) = sink_entities.choose(&mut rng) {
        // Pick a random contract definition
        if let Some(contract_bundle) = find_and_generate_contract(**faction, **reputation, &contract_library) {
            let contract_entity = commands.spawn(contract_bundle).id();
            commands.entity(contract_entity).insert(AssociatedWithSink(*sink_entity));
            info!("Generated new pending contract {:?} for sink {:?}", contract_entity, sink_entity);
        } else {
            info!("No suitable contract found for sink {:?} with faction {:?} and reputation {:?}", sink_entity, faction, reputation);
        }
    } else {
        info!("No unlocked and free SinkBuilding found to assign a new contract.");
    }
}

// Startup system to load the contracts.ron file
fn load_contracts_from_ron(mut commands: Commands) {
    // Read the file from the assets folder.
    let ron_str = std::fs::read_to_string("assets/text/contracts.ron")
        .expect("Failed to read contracts.ron");

    // Parse the RON string into a Vec first, then collect into a HashMap by id
    #[derive(Debug, serde::Deserialize)]
    struct RonContractsList {
        contracts: Vec<ContractDefinition>,
    }
    let contracts_list: RonContractsList = ron::from_str(&ron_str)
        .expect("Failed to parse contracts from RON");
    let contracts = contracts_list.contracts.into_iter().map(|c| {
        (c.id, c)
    }).collect();
    let contract_library = ContractLibrary { contracts };

    // Insert the fully loaded data as a Bevy Resource.
    commands.insert_resource(contract_library);
    info!("Contracts loaded and inserted as a Resource.");
}

/// A test system to verify contract generation logic at startup.
fn test_find_and_generate_contract(library: Res<ContractLibrary>, mut commands: Commands) {
    let faction_corporate = Faction::Academia;
    let reputation = ReputationLevel::Neutral;

    if let Some(mut contract_bundle) =
        find_and_generate_contract(faction_corporate, reputation, &library)
    {
        info!(
            "  -> SUCCESS: Found contract '{:?}'", contract_bundle
        );
        contract_bundle.status = ContractStatus::Active;
        contract_bundle.fulfillment_info.update_throughput(49.0);
        commands.spawn(contract_bundle);
    } else {
        info!("  -> FAILURE: No contract found for Corporate faction reputation.");
    }

    if let Some(mut contract_bundle) =
        find_and_generate_contract(faction_corporate, reputation, &library)
    {
        info!(
            "  -> SUCCESS: Found contract '{:?}'", contract_bundle
        );
        contract_bundle.status = ContractStatus::Active;
        contract_bundle.fulfillment_info.update_throughput(75.0);
        commands.spawn(contract_bundle);
    } else {
        info!("  -> FAILURE: No contract found for Corporate faction reputation.");
    }

    if let Some(mut contract_bundle) =
        find_and_generate_contract(faction_corporate, reputation, &library)
    {
        info!(
            "  -> SUCCESS: Found contract '{:?}'", contract_bundle
        );
        contract_bundle.status = ContractStatus::Active;
        contract_bundle.fulfillment_info.update_throughput(125.0);
        commands.spawn(contract_bundle);
    } else {
        info!("  -> FAILURE: No contract found for Corporate faction reputation.");
    }
}

// --- Contract Generation Logic ---

/// Finds a suitable contract from the library for a given sink.
pub fn find_and_generate_contract(
    sink_faction: Faction,
    sink_reputation: ReputationLevel,
    library: &ContractLibrary,
) -> Option<ContractBundle> {
    // Find an available contract that matches the sink's faction and reputation
    let suitable_contract = library.all_contracts().into_iter().find(|c| {
        c.faction == sink_faction && sink_reputation >= c.reputation
    })?;

    // Use the found contract definition to create a ContractBundle
    Some(ContractBundle {
        contract: Contract,
        status: ContractStatus::Pending,
        dataset: suitable_contract.dataset.clone(),
        faction: suitable_contract.faction.clone(),
        timeout: ContractTimeout(120.), // Default timeout
        description: ContractDescription {
            name: suitable_contract.name.clone(),
            description: suitable_contract.description.clone(),
        },
        fulfillment_info: ContractFulfillment::new(
            suitable_contract.base_threshold, 
            suitable_contract.base_money
        ),
    })
}

/// System to handle contracts that have been failing for too long
fn handle_failing_contract_timeouts(
    mut commands: Commands,
    time: Res<Time>,
    mut contracts: Query<(
        Entity,
        &mut ContractStatus,
        &ContractFulfillment,
        &ContractTimeout,
        Option<&mut FailingTimer>,
    )>,
) {
    for (entity, mut status, fulfillment, timeout, failing_timer) in contracts.iter_mut() {
        // Skip non-active contracts
        if *status != ContractStatus::Active {
            continue;
        }

        match fulfillment.status {
            ContractFulfillmentStatus::Failing => {
                // Contract is currently failing
                if let Some(mut timer) = failing_timer {
                    // Update existing timer
                    timer.timer.tick(time.delta());
                    if timer.timer.is_finished() {
                        // Timer expired, fail the contract
                        *status = ContractStatus::Failed;
                        commands.entity(entity).remove::<FailingTimer>();
                        info!("Contract {:?} failed due to timeout after {:.1}s of failing", entity, timeout.0);
                    }
                } else {
                    // Start the failing timer
                    let failing_timer = FailingTimer::new(timeout.0);
                    commands.entity(entity).insert(failing_timer);
                    info!("Contract {:?} started failing timer ({:.1}s)", entity, timeout.0);
                }
            }
            ContractFulfillmentStatus::Meeting | ContractFulfillmentStatus::Exceeding => {
                // Contract is meeting requirements, remove failing timer if present
                if failing_timer.is_some() {
                    commands.entity(entity).remove::<FailingTimer>();
                    info!("Contract {:?} recovered from failing state", entity);
                }
            }
        }
    }
}

fn get_fulfillment_status(threshold_fraction: f64) -> ContractFulfillmentStatus {
    if threshold_fraction >= 2.0 {
        ContractFulfillmentStatus::Exceeding
    } else if threshold_fraction >= 1.0 {
        ContractFulfillmentStatus::Meeting
    } else {
        ContractFulfillmentStatus::Failing
    }
}