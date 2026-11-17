use crate::spatial::wrapped_delta;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::genome::Genome;
use crate::neural_controller::NeuralAction;
use crate::neural_genome::NeuralGenome;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Species {
    #[default]
    Herbivore,
    Predator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organism {
    pub id: u64,
    #[serde(default)]
    pub species: Species,
    pub parent_id: Option<u64>,
    pub generation: u32,

    pub x: f32,
    pub y: f32,

    pub vx: f32,
    pub vy: f32,

    pub energy: f32,
    #[serde(skip)]
    pub previous_energy: f32,
    pub age: u32,

    pub genome: Genome,
    #[serde(skip)]
    pub brain: NeuralGenome,
}

impl Organism {
    pub fn new(id: u64, config: &SimulationConfig, rng: &mut impl Rng) -> Self {
        let genome = Genome::random(rng);

        let angle = rng.random_range(0.0..std::f32::consts::TAU);

        Self {
            id,
            species: Species::Herbivore,
            parent_id: None,
            generation: 0,

            x: rng.random_range(0.0..config.width),
            y: rng.random_range(0.0..config.height),

            vx: angle.cos() * genome.speed,
            vy: angle.sin() * genome.speed,

            energy: config.initial_energy,
            previous_energy: config.initial_energy,
            age: 0,

            genome,
            brain: NeuralGenome::random(config.controller, rng),
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
        let turn = rng.random_range(-0.3..0.3);

        self.wander_with_turn(turn);
    }

    pub fn wander_with_turn(&mut self, turn: f32) {
        let current_angle = self.vy.atan2(self.vx);

        let new_angle = current_angle + turn;

        self.vx = new_angle.cos() * self.genome.speed;
        self.vy = new_angle.sin() * self.genome.speed;
    }

    pub fn update(&mut self, config: &SimulationConfig) {
        self.x = (self.x + self.vx).rem_euclid(config.width);
        self.y = (self.y + self.vy).rem_euclid(config.height);

        self.age += 1;

        let speed_cost = 0.02 * (self.vx * self.vx + self.vy * self.vy);
        let sensor_cost = self.genome.sensor_radius * 0.001;
        let size_cost = self.genome.size * 0.01;

        let energy_cost = self.genome.metabolism + speed_cost + sensor_cost + size_cost;

        let scale = if config.predation && self.species == Species::Predator {
            config.predator_metabolism_scale
        } else {
            1.0
        };
        self.energy = (self.energy - energy_cost * scale).max(0.0);
        if config.hazard_zones {
            let dx = wrapped_delta(self.x, config.width * 0.5, config.width);
            let dy = wrapped_delta(self.y, config.height * 0.5, config.height);
            let radius = config.width.min(config.height) * 0.12;
            if dx * dx + dy * dy < radius * radius {
                self.energy = (self.energy - config.hazard_damage).max(0.0);
            }
        }
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
            species: self.species,
            parent_id: Some(self.id),
            generation: self.generation + 1,

            x: (self.x + offset_x).rem_euclid(config.width),
            y: (self.y + offset_y).rem_euclid(config.height),

            vx: angle.cos() * child_genome.speed,
            vy: angle.sin() * child_genome.speed,
            energy: child_energy,
            previous_energy: child_energy,
            age: 0,

            genome: child_genome,
            brain: self.brain.inherit(config.neural_mutation(), rng),
        })
    }

    pub fn has_neural_controller(&self) -> bool {
        self.brain.is_neural()
    }

    pub fn neural_parameter_count(&self) -> usize {
        self.brain.parameter_count()
    }

    pub fn neural_parameter_bytes(&self) -> usize {
        self.brain.parameter_bytes()
    }
    pub fn record_energy(&mut self) {
        self.previous_energy = self.energy;
    }

    pub fn apply_neural_action(&mut self, action: NeuralAction) {
        let angle = self.vy.atan2(self.vx) + action.turn;
        let speed = self.genome.speed * action.speed;

        self.vx = angle.cos() * speed;
        self.vy = angle.sin() * speed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural_genome::ControllerKind;
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

    #[test]
    fn organism_defaults_to_heuristic_controller() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let organism = Organism::new(0, &config, &mut rng);

        assert!(!organism.has_neural_controller());
        assert_eq!(organism.brain.kind(), ControllerKind::Heuristic);
        assert_eq!(organism.neural_parameter_count(), 0);
    }

    #[test]
    fn organism_initializes_small_controller() {
        let config = SimulationConfig {
            controller: ControllerKind::Small,
            ..SimulationConfig::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let organism = Organism::new(0, &config, &mut rng);

        assert!(organism.has_neural_controller());
        assert_eq!(organism.brain.kind(), ControllerKind::Small);
        assert_eq!(organism.neural_parameter_count(), 83);
        assert_eq!(organism.neural_parameter_bytes(), 332);
    }

    #[test]
    fn organism_initializes_deep_controller() {
        let config = SimulationConfig {
            controller: ControllerKind::Deep,
            ..SimulationConfig::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let organism = Organism::new(0, &config, &mut rng);

        assert!(organism.has_neural_controller());
        assert_eq!(organism.brain.kind(), ControllerKind::Deep);
        assert_eq!(organism.neural_parameter_count(), 1140);
        assert_eq!(organism.neural_parameter_bytes(), 4560);
    }

    #[test]
    fn offspring_inherits_neural_controller() {
        let config = SimulationConfig {
            controller: ControllerKind::Deep,
            neural_mutation_rate: 0.0,
            ..SimulationConfig::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut parent = Organism::new(0, &config, &mut rng);

        parent.energy = config.max_energy;
        parent.age = 100;

        let original_brain = parent.brain.clone();
        let child = parent.reproduce(1, &config, &mut rng).unwrap();

        assert_eq!(child.brain.kind(), ControllerKind::Deep);
        assert_eq!(child.brain, original_brain);
        assert_eq!(parent.brain, original_brain);
    }

    #[test]
    fn offspring_brain_mutates_independently() {
        let config = SimulationConfig {
            controller: ControllerKind::Deep,
            neural_mutation_rate: 1.0,
            neural_mutation_strength: 0.1,
            ..SimulationConfig::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut parent = Organism::new(0, &config, &mut rng);

        parent.energy = config.max_energy;
        parent.age = 100;

        let original_brain = parent.brain.clone();
        let child = parent.reproduce(1, &config, &mut rng).unwrap();

        assert_eq!(parent.brain, original_brain);
        assert_ne!(child.brain, original_brain);
        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.generation, parent.generation + 1);
    }

    #[test]
    fn organism_snapshot_excludes_brain_parameters() {
        let config = SimulationConfig {
            controller: ControllerKind::Deep,
            ..SimulationConfig::default()
        };

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let organism = Organism::new(0, &config, &mut rng);

        let snapshot = serde_json::to_value(&organism).unwrap();

        assert!(snapshot.get("brain").is_none());
        assert!(snapshot.get("genome").is_some());
    }
}
