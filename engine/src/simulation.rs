use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::config::SimulationConfig;
use crate::metrics::Metrics;
use crate::spatial::distance_squared;
use crate::world::World;

pub struct Simulation {
    pub config: SimulationConfig,
    pub world: World,

    pub tick: u64,
    pub total_births: u64,
    pub total_deaths: u64,

    rng: ChaCha8Rng,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
        let world = World::new(&config, &mut rng);

        Self {
            config,
            world,
            tick: 0,
            total_births: 0,
            total_deaths: 0,
            rng,
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;

        self.spawn_food();
        self.move_organisms();
        self.feed_organisms();
        self.reproduce_organisms();
        self.remove_dead();
    }

    fn spawn_food(&mut self) {
        self.world
            .spawn_food(self.config.food_spawn_rate, &self.config, &mut self.rng);
    }

    fn move_organisms(&mut self) {
        let food = &self.world.food;
        let config = &self.config;

        for organism in &mut self.world.organisms {
            if !organism.is_alive(config) {
                continue;
            }

            let mut nearest_food = None;

            let mut nearest_distance = organism.genome.sensor_radius.powi(2);

            for resource in food {
                let distance = distance_squared(
                    organism.x,
                    organism.y,
                    resource.x,
                    resource.y,
                    config.width,
                    config.height,
                );

                if distance < nearest_distance {
                    nearest_distance = distance;

                    nearest_food = Some((resource.x, resource.y));
                }
            }

            match nearest_food {
                Some((x, y)) => {
                    organism.move_towards(x, y, config);
                }

                None => {
                    organism.wander(&mut self.rng);
                }
            }

            organism.update(config);
        }
    }

    fn feed_organisms(&mut self) {
        let config = &self.config;

        let mut consumed = vec![false; self.world.food.len()];

        for organism in &mut self.world.organisms {
            if !organism.is_alive(config) {
                continue;
            }

            if organism.energy >= config.max_energy {
                continue;
            }

            let eating_radius = organism.genome.size + 2.0;
            let mut nearest_distance = eating_radius.powi(2);
            let mut nearest_index = None;

            for (index, food) in self.world.food.iter().enumerate() {
                if consumed[index] {
                    continue;
                }

                let distance = distance_squared(
                    organism.x,
                    organism.y,
                    food.x,
                    food.y,
                    config.width,
                    config.height,
                );

                if distance <= nearest_distance {
                    nearest_distance = distance;
                    nearest_index = Some(index);
                }
            }

            if let Some(index) = nearest_index {
                let energy = self.world.food[index].energy;

                organism.eat(energy, config);
                consumed[index] = true;
            }
        }

        let mut index = 0;

        self.world.food.retain(|_| {
            let keep = !consumed[index];
            index += 1;
            keep
        });
    }

    fn reproduce_organisms(&mut self) {
        let available_space = self
            .config
            .max_population
            .saturating_sub(self.world.organisms.len());

        let mut offspring = Vec::new();
        let mut next_id = self.world.next_organism_id;

        for organism in &mut self.world.organisms {
            if offspring.len() >= available_space {
                break;
            }

            if let Some(child) = organism.reproduce(next_id, &self.config, &mut self.rng) {
                offspring.push(child);
                next_id += 1;
            }
        }

        self.total_births += offspring.len() as u64;

        self.world.next_organism_id = next_id;
        self.world.organisms.extend(offspring);
    }

    fn remove_dead(&mut self) {
        let before = self.world.organisms.len();

        self.world
            .organisms
            .retain(|organism| organism.is_alive(&self.config));

        let deaths = before - self.world.organisms.len();

        self.total_deaths += deaths as u64;
    }

    pub fn run(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.step();
        }
    }

    pub fn metrics(&self) -> Metrics {
        Metrics::collect(&self.world, self.tick, self.total_births, self.total_deaths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_advances_ticks() {
        let mut sim = Simulation::new(SimulationConfig::default());

        sim.run(10);

        assert_eq!(sim.tick, 10);
    }

    #[test]
    fn population_respects_limit() {
        let config = SimulationConfig {
            max_population: 50,
            initial_population: 50,
            ..SimulationConfig::default()
        };

        let mut sim = Simulation::new(config);

        sim.run(100);

        assert!(sim.world.organisms.len() <= 50);
    }

    #[test]
    fn simulation_is_reproducible() {
        let config = SimulationConfig::default();

        let mut sim_a = Simulation::new(config.clone());
        let mut sim_b = Simulation::new(config);

        sim_a.run(100);
        sim_b.run(100);

        let state_a = serde_json::to_string(&sim_a.world).unwrap();
        let state_b = serde_json::to_string(&sim_b.world).unwrap();

        assert_eq!(state_a, state_b);
    }

    #[test]
    fn population_accounting_is_correct() {
        let config = SimulationConfig::default();
        let mut sim = Simulation::new(config);

        let initial_population = sim.world.organisms.len() as u64;

        sim.run(200);

        assert_eq!(
            sim.world.organisms.len() as u64,
            initial_population + sim.total_births - sim.total_deaths
        );
    }

    #[test]
    fn empty_population_has_valid_metrics() {
        let config = SimulationConfig {
            initial_population: 0,
            ..SimulationConfig::default()
        };

        let sim = Simulation::new(config);
        let metrics = sim.metrics();

        assert_eq!(metrics.population, 0);
        assert_eq!(metrics.average_energy, 0.0);
        assert_eq!(metrics.average_speed, 0.0);
    }
}
