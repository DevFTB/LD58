use bevy::{prelude::*};
use bevy::ecs::relationship::{RelationshipTarget};
use serde::Deserialize;
use crate::factory::logical::{Dataset};
use crate::factions::Faction;
use bevy::platform::collections::HashMap;

// Add the Deserialize trait to your existing components that are in the RON file
#[derive(Component, Deserialize, Debug)]
pub struct Contract;

#[derive(Component, Deserialize, Debug)]
pub struct ContractTimeout(pub f32);

#[derive(Component, Deserialize, Debug)]
pub enum ContractStatus {
    Pending,
    Active,
    Completed,
}

#[derive(Component, Deserialize, Debug)]
pub struct ContractThroughput(f64);

#[derive(Component, Deserialize, Debug)]
pub struct ContractMoneyRate(f64);


#[derive(Component, Default, Deserialize, Clone, Debug)]
pub struct ContractDescription {
    pub name: String,
    pub description: String,
}

// --- New Structs for RON loading ---

// Represents a single contract definition from the RON file
#[derive(Debug, Deserialize, Clone)]
pub struct ContractDefinition {
    pub name: String,
    pub description: String,
    pub faction: Faction,
    pub reputation: i32,
    pub throughput: f64,
    pub money_rate: f64,
    pub dataset: Dataset,
}

// A resource to hold all contracts loaded from the RON file
#[derive(Resource, Debug, Deserialize, Default)]
pub struct ContractLibrary {
    pub contracts: Vec<ContractDefinition>,
}

// --- Plugin and Systems ---

pub struct ContractsPlugin;

impl Plugin for ContractsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_contracts_from_ron)
            .add_systems(Startup, test_find_and_generate_contract);
    }
}

// Startup system to load the contracts.ron file
fn load_contracts_from_ron(mut commands: Commands) {
    // Read the file from the assets folder.
    let ron_str = std::fs::read_to_string("assets/config/contracts.ron")
        .expect("Failed to read contracts.ron");

    // Parse the RON string into our ContractLibrary struct.
    let contract_library: ContractLibrary = ron::from_str(&ron_str)
        .expect("Failed to parse contracts from RON");

    // Insert the fully loaded data as a Bevy Resource.
    commands.insert_resource(contract_library);
    info!("Contracts loaded and inserted as a Resource.");
}

/// A test system to verify contract generation logic at startup.
fn test_find_and_generate_contract(library: Res<ContractLibrary>, mut commands: Commands) {
    let faction_corporate = Faction::Academia;
    let reputation = 1;

    if let Some(contract_bundle) =
        find_and_generate_contract(faction_corporate, reputation, &library)
    {
        info!(
            "  -> SUCCESS: Found contract '{:?}'", contract_bundle
        );
        commands.spawn(contract_bundle);
    } else {
        info!("  -> FAILURE: No contract found for Corporate faction reputation.");
    }
}

// --- Contract Generation Logic ---

/// Finds a suitable contract from the library for a given sink.
pub fn find_and_generate_contract(
    sink_faction: Faction,
    sink_reputation: i32,
    library: &ContractLibrary,
) -> Option<ContractBundle> {
    // Find an available contract that matches the sink's faction and reputation
    let suitable_contract = library.contracts.iter().find(|c| {
        c.faction == sink_faction && sink_reputation >= c.reputation
    })?;

    // Use the found contract definition to create a ContractBundle
    Some(ContractBundle {
        contract: Contract,
        status: ContractStatus::Pending,
        dataset: suitable_contract.dataset.clone(),
        throughput: ContractThroughput(suitable_contract.throughput),
        money_rate: ContractMoneyRate(suitable_contract.money_rate),
        faction: suitable_contract.faction.clone(),
        timeout: ContractTimeout(120.0), // Default timeout
        description: ContractDescription {
            name: suitable_contract.name.clone(),
            description: suitable_contract.description.clone(),
        },
    })
}

// --- Existing Relationship Structs ---
#[derive(Component)]
#[relationship(relationship_target = SinkContracts)]
pub struct AssociatedWithSink(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = AssociatedWithSink)]
pub struct SinkContracts(Vec<Entity>);

impl SinkContracts {
    pub fn contracts(&self) -> &[Entity] {
        &self.0
    }
}

#[derive(Bundle, Debug)]
pub struct ContractBundle {
    pub contract: Contract,
    pub status: ContractStatus,
    pub dataset: Dataset,
    pub throughput: ContractThroughput,
    pub money_rate: ContractMoneyRate,
    pub faction: Faction,
    pub timeout: ContractTimeout,
    pub description: ContractDescription,
}
