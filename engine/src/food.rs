use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Food {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub energy: f32,
}

impl Food {
    pub fn new(id: u64, config: &SimulationConfig, rng: &mut impl Rng) -> Self {
        Self {
            id,
            x: rng.random_range(0.0..config.width),
            y: rng.random_range(0.0..config.height),
            energy: config.food_energy,
        }
    }
}
