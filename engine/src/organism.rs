use crate::spatial::wrapped_delta;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::genome::Genome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organism {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub generation: u32,

    pub x: f32,
    pub y: f32,

    pub vx: f32,
    pub vy: f32,

    pub energy: f32,
    pub age: u32,

    pub genome: Genome,
}

impl Organism {
    pub fn new(id: u64, config: &SimulationConfig, rng: &mut impl Rng) -> Self {
        let genome = Genome::random(rng);

        let angle = rng.random_range(0.0..std::f32::consts::TAU);

        Self {
            id,
            parent_id: None,
            generation: 0,

            x: rng.random_range(0.0..config.width),
            y: rng.random_range(0.0..config.height),

            vx: angle.cos() * genome.speed,
            vy: angle.sin() * genome.speed,

            energy: config.initial_energy,
            age: 0,

            genome,
        }
    }

    pub fn move_towards(&mut self, target_x: f32, target_y: f32, config: &SimulationConfig) {
        let dx = wrapped_delta(self.x, target_x, config.width);
        let dy = wrapped_delta(self.y, target_y, config.height);

        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.001 {
            let speed = self.genome.speed.min(distance);

            self.vx = dx / distance * speed;
            self.vy = dy / distance * speed;
        } else {
            self.vx = 0.0;
            self.vy = 0.0;
        }
    }

    pub fn wander(&mut self, rng: &mut impl Rng) {
        let current_angle = self.vy.atan2(self.vx);
        let turn = rng.random_range(-0.3..0.3);

        let angle = current_angle + turn;

        self.vx = angle.cos() * self.genome.speed;
        self.vy = angle.sin() * self.genome.speed;
    }

    pub fn update(&mut self, config: &SimulationConfig) {
        self.x = (self.x + self.vx).rem_euclid(config.width);
        self.y = (self.y + self.vy).rem_euclid(config.height);

        self.age += 1;

        let speed_cost = self.genome.speed.powi(2) * 0.02;
        let sensor_cost = self.genome.sensor_radius * 0.001;
        let size_cost = self.genome.size * 0.01;

        let energy_cost = self.genome.metabolism + speed_cost + sensor_cost + size_cost;

        self.energy = (self.energy - energy_cost).max(0.0);
    }

    pub fn eat(&mut self, energy: f32, config: &SimulationConfig) {
        self.energy = (self.energy + energy).min(config.max_energy);
    }

    pub fn is_alive(&self, config: &SimulationConfig) -> bool {
        self.energy > 0.0 && self.age < config.max_age
    }

    pub fn can_reproduce(&self) -> bool {
        self.energy >= self.genome.reproduction_threshold && self.age >= 20
    }

    pub fn reproduce(
        &mut self,
        child_id: u64,
        config: &SimulationConfig,
        rng: &mut impl Rng,
    ) -> Option<Self> {
        if !self.can_reproduce() || !self.is_alive(config) {
            return None;
        }

        let child_genome = self
            .genome
            .mutate(rng, config.mutation_rate, config.mutation_strength);

        let child_energy = self.energy * 0.4;
        self.energy -= child_energy;

        let angle = rng.random_range(0.0..std::f32::consts::TAU);

        let offset_x = rng.random_range(-10.0..10.0);
        let offset_y = rng.random_range(-10.0..10.0);

        Some(Self {
            id: child_id,
            parent_id: Some(self.id),
            generation: self.generation + 1,

            x: (self.x + offset_x).rem_euclid(config.width),
            y: (self.y + offset_y).rem_euclid(config.height),

            vx: angle.cos() * child_genome.speed,
            vy: angle.sin() * child_genome.speed,

            energy: child_energy,
            age: 0,

            genome: child_genome,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn organism_loses_energy_and_ages() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut organism = Organism::new(0, &config, &mut rng);
        let initial_energy = organism.energy;

        organism.update(&config);

        assert!(organism.energy < initial_energy);
        assert_eq!(organism.age, 1);
    }

    #[test]
    fn eating_respects_energy_limit() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut organism = Organism::new(0, &config, &mut rng);

        organism.eat(1000.0, &config);

        assert_eq!(organism.energy, config.max_energy);
    }

    #[test]
    fn organism_dies_without_energy() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut organism = Organism::new(0, &config, &mut rng);
        organism.energy = 0.0;

        assert!(!organism.is_alive(&config));
    }

    #[test]
    fn reproduction_conserves_energy() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut parent = Organism::new(0, &config, &mut rng);

        parent.energy = 250.0;
        parent.age = 100;

        let initial_energy = parent.energy;

        let child = parent.reproduce(1, &config, &mut rng).unwrap();

        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.generation, parent.generation + 1);
        assert_eq!(child.age, 0);

        assert!((parent.energy + child.energy - initial_energy).abs() < 0.001);
    }

    #[test]
    fn world_boundary_wraps() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut organism = Organism::new(0, &config, &mut rng);

        organism.x = config.width - 1.0;
        organism.vx = 3.0;

        organism.update(&config);

        assert!((organism.x - 2.0).abs() < 0.001);
    }
}
