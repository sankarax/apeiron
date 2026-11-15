use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::SimulationConfig;
use crate::food::Food;
use crate::organism::Organism;

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

            world.organisms.push(Organism::new(id, config, rng));
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
            self.food.push(Food::new(self.next_food_id, config, rng));

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
