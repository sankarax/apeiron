use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub width: f32,
    pub height: f32,

    pub initial_population: usize,
    pub max_population: usize,

    pub initial_food: usize,
    pub max_food: usize,
    pub food_spawn_rate: usize,
    pub food_energy: f32,

    pub initial_energy: f32,
    pub max_energy: f32,
    pub max_age: u32,

    pub mutation_rate: f32,
    pub mutation_strength: f32,

    pub seed: u64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            width: 1600.0,
            height: 900.0,

            initial_population: 200,
            max_population: 5000,

            initial_food: 500,
            max_food: 2000,
            food_spawn_rate: 10,
            food_energy: 25.0,

            initial_energy: 100.0,
            max_energy: 250.0,
            max_age: 3000,

            mutation_rate: 0.1,
            mutation_strength: 0.15,

            seed: 42,
        }
    }
}
