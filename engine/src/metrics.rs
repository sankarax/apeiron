use serde::{Deserialize, Serialize};

use crate::world::World;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    pub tick: u64,

    pub population: usize,
    pub food_count: usize,

    pub total_births: u64,
    pub total_deaths: u64,

    pub average_energy: f32,
    pub average_age: f32,

    pub average_speed: f32,
    pub average_sensor_radius: f32,
    pub average_metabolism: f32,
    pub average_size: f32,
    pub average_reproduction_threshold: f32,

    pub average_generation: f32,
    pub max_generation: u32,
}

impl Metrics {
    pub fn collect(world: &World, tick: u64, total_births: u64, total_deaths: u64) -> Self {
        let population = world.organisms.len();

        let mut metrics = Self {
            tick,
            population,
            food_count: world.food.len(),
            total_births,
            total_deaths,
            ..Self::default()
        };

        if population == 0 {
            return metrics;
        }

        for organism in &world.organisms {
            metrics.average_energy += organism.energy;
            metrics.average_age += organism.age as f32;

            metrics.average_speed += organism.genome.speed;
            metrics.average_sensor_radius += organism.genome.sensor_radius;
            metrics.average_metabolism += organism.genome.metabolism;
            metrics.average_size += organism.genome.size;
            metrics.average_reproduction_threshold += organism.genome.reproduction_threshold;

            metrics.average_generation += organism.generation as f32;

            metrics.max_generation = metrics.max_generation.max(organism.generation);
        }

        let n = population as f32;

        metrics.average_energy /= n;
        metrics.average_age /= n;
        metrics.average_speed /= n;
        metrics.average_sensor_radius /= n;
        metrics.average_metabolism /= n;
        metrics.average_size /= n;
        metrics.average_reproduction_threshold /= n;
        metrics.average_generation /= n;

        metrics
    }
}
