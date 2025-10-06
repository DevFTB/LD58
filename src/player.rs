use bevy::prelude::*;

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