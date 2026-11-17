use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::food::Food;
use crate::organism::{Organism, Species};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub width: f32,
    pub height: f32,

    pub organisms: Vec<Organism>,
    pub food: Vec<Food>,

    pub next_organism_id: u64,
    pub next_food_id: u64,
}

impl World {
    pub fn new(config: &SimulationConfig, rng: &mut impl Rng) -> Self {
        let mut world = Self {
            width: config.width,
            height: config.height,

            organisms: Vec::new(),
            food: Vec::new(),

            next_organism_id: 0,
            next_food_id: 0,
        };

        for _ in 0..config.initial_population.min(config.max_population) {
            let id = world.allocate_organism_id();

            let mut organism = Organism::new(id, config, rng);
            if config.predation && rng.random::<f32>() < config.predator_fraction {
                organism.species = Species::Predator;
            }
            world.organisms.push(organism);
        }

        world.spawn_food(config.initial_food, config, rng);

        world
    }

    pub fn allocate_organism_id(&mut self) -> u64 {
        let id = self.next_organism_id;
        self.next_organism_id += 1;
        id
    }

    pub fn spawn_food(&mut self, count: usize, config: &SimulationConfig, rng: &mut impl Rng) {
        let available_space = config.max_food.saturating_sub(self.food.len());

        let spawn_count = count.min(available_space);

        for _ in 0..spawn_count {
            let mut food = Food::new(self.next_food_id, config, rng);
            if config.resource_zones && rng.random::<f32>() < 0.8 {
                let center = if rng.random::<bool>() { 0.25 } else { 0.75 };
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let radius = rng.random::<f32>().sqrt() * config.width.min(config.height) * 0.15;
                food.x = (config.width * center + radius * angle.cos()).rem_euclid(config.width);
                food.y = (config.height * 0.5 + radius * angle.sin()).rem_euclid(config.height);
            }
            self.food.push(food);

            self.next_food_id += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn world_initializes_correctly() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let world = World::new(&config, &mut rng);

        assert_eq!(world.organisms.len(), config.initial_population);

        assert_eq!(world.food.len(), config.initial_food);
    }

    #[test]
    fn food_respects_population_limit() {
        let config = SimulationConfig::default();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut world = World::new(&config, &mut rng);

        world.spawn_food(10000, &config, &mut rng);

        assert_eq!(world.food.len(), config.max_food);
    }
}
