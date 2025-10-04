use bevy::{
    color::Color,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{Added, Or, With, Without},
        system::{Commands, Query},
    },
};

use crate::{
    factory::logical::{DataInput, DataOutput, LogicalLink},
    grid::{Direction, GridPosition, GridSprite},
};

#[derive(Component)]
pub struct PhysicalInput(Entity, Direction);

#[derive(Component)]
pub struct PhysicalOutput(Entity, Direction);

#[derive(Component)]
pub struct PhysicalLink {
    pub logical_link: Option<Entity>,
    pub throughput: f32,
}
impl PhysicalLink {
    pub fn get_spawn_bundle(position: GridPosition) -> impl Bundle {
        (
            position,
            PhysicalLink {
                logical_link: None,
                throughput: 234.,
            },
            GridSprite(Color::linear_rgba(0.0, 0.0, 1.0, 1.0)),
        )
    }
}

pub fn connect_physical_inputs(
    query: Query<(Entity, &GridPosition), Added<PhysicalLink>>,
    mut commands: Commands,
    inputs: Query<
        (
            Entity,
            &GridPosition,
            &DataInput
        ),
        Without<PhysicalInput>,
    >,
) {
    for (entity, new_grid_position) in query.iter() {
        let neighbours = new_grid_position.neighbours();
        // Determine directionality in input
        let candidate = inputs.iter()
            .filter_map(|(input_entity,grid_pos,input)|
                neighbours.iter()
                    .find(|(dir, pos)| grid_pos == pos && **input == dir.opposite())
                    .map(|(dir, _)| (input_entity, dir))
            ).next();

        if let Some((neighbour_entity, dir)) = candidate {
            insert_physical_connection(&mut commands, entity, neighbour_entity, &dir);
        }
    }
}

pub fn connect_physical_outputs(
    query: Query<(Entity, &GridPosition), Added<PhysicalLink>>,
    mut commands: Commands,
    inputs: Query<
        (
            Entity,
            &GridPosition,
            &DataOutput
        ),
        Without<PhysicalOutput>,
    >,
) {

    for (entity, new_grid_position) in query.iter() {
        let neighbours = new_grid_position.neighbours();
        // Determine directionality in input
        let candidate = inputs.iter()
            .filter_map(|(input_entity,grid_pos,input)|
                neighbours.iter()
                    .find(|(dir, pos)| grid_pos == pos && **input == dir.opposite())
                    .map(|(dir, _)| (input_entity, dir))
            ).next();

        if let Some((neighbour_entity, dir)) = candidate {
            insert_physical_connection(&mut commands, neighbour_entity, entity, &dir);
        }
    }
}


pub fn connect_links(
    mut commands: Commands,
    new_links: Query<Entity, Added<PhysicalLink>>,
    links: Query<
        (
            Entity,
            &GridPosition,
        ),
        With<PhysicalLink>
    >,
    connections: Query<(Option<&PhysicalInput>, Option<&PhysicalOutput>)
    >,
) {
    for entity in new_links.iter() {
        let (entity, new_grid_position) = links.get(entity).unwrap();
        // Determine directionality in input and output
        let possible_neighbours = new_grid_position.neighbours();

        let neighbours = links
            .iter()
            .filter_map(|(neighbour_entity, position)| {
                possible_neighbours
                    .iter()
                    .find(|n| n.1 == *position)
                    .map(|(dir, _)| (neighbour_entity, dir))
            })
            .collect::<Vec<_>>();

        let first_open_neighbour_input = neighbours.iter()
            .map(|(neighbour_entity, dir)| (neighbour_entity,dir))
            .find(|&(&entity,_dir)| connections.get(entity).unwrap().0.is_none());

        if let Some((neighbour_entity, dir)) = first_open_neighbour_input {
            insert_physical_connection(&mut commands, entity, *neighbour_entity, dir);
        }
        let first_open_neighbour_output = neighbours.iter()
            .filter(|(ne, _)| first_open_neighbour_input.map(|(fne, _)| fne != ne).unwrap_or(false))
            .map(|(neighbour_entity, dir)| (neighbour_entity,dir))
            .find(|&(&entity,_dir)| connections.get(entity).unwrap().0.is_none());

        if let Some((neighbour_entity, dir)) = first_open_neighbour_output{
            insert_physical_connection(&mut commands, *neighbour_entity, entity, dir);
        }
    }
}

fn insert_physical_connection(commands: &mut Commands, output_entity: Entity, input_entity: Entity, dir: &&Direction) {
    commands
        .entity(output_entity)
        .insert(PhysicalOutput(input_entity, **dir));
    println!("PhysicalOutput on {:?} to {:?}", output_entity, input_entity);

    commands
        .entity(input_entity)
        .insert(PhysicalInput(output_entity, dir.opposite()));
    println!("PhysicalInput on {:?} to {:?}", input_entity, output_entity);
}

pub fn establish_logical_links(
    query: Query<Entity, Added<PhysicalLink>>,
    mut commands: Commands,
    inputs: Query<&PhysicalInput>,
    outputs: Query<&PhysicalOutput>,
    data_inputs: Query<Entity, With<DataInput>>,
    data_outputs: Query<Entity, With<DataOutput>>,
    links: Query<&PhysicalLink>
) {

    for entity in query.iter() {
        if let (Ok(PhysicalInput(next_input, _)), Ok(PhysicalOutput(next_output, _))) =
            (inputs.get(entity), outputs.get(entity))
        {
            //Traverse input linked list
            let Some((data_input_entity, mut output_links)) = ({
                let mut links: Vec<Entity> = Vec::new();
                links.push(*next_input);
                while let Ok(next) = inputs.get(*links.last().unwrap()) {
                    println!("{:?}", links);
                    println!("Entity {:?} is after {:?}", next.0, *links.last().unwrap());
                    links.push(next.0);
                }

                Some((links.pop().unwrap(), links))
            }) else {
                return;
            };

            let Some((data_output_entity, mut input_links)) = ({
                let mut links: Vec<Entity> = Vec::new();
                links.push(*next_output);
                while let Ok(next) = outputs.get(*links.last().unwrap()) {
                    links.push(next.0);
                }

                Some((links.pop().unwrap(), links))
            }) else {
                return;
            };

            let mut full_links = Vec::<Entity>::new();
            full_links.append(&mut input_links);
            full_links.push(entity);
            full_links.append(&mut output_links);

            let throughput = full_links.iter().map(|e| links.get(*e).unwrap().throughput).reduce(f32::min)
                .unwrap_or(0.);

            let link = LogicalLink { links: full_links, throughput, output_entity: data_output_entity, input_entity: data_input_entity };
            commands
                .entity(data_input_entity)
                .insert(link);

            println!("FUCK YEAH!!!")
        }
    }
}
