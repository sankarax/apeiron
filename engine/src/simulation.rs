use crate::food_search::{
    nearest_edible_food_brute_force, nearest_edible_food_spatial, nearest_food_brute_force,
    nearest_food_spatial,
};

use crate::compute_backend::ComputeBackend;
use crate::parallel::{MovementAction, execute_movement_parallel, sense_food_parallel};
use crate::spatial_grid::SpatialGrid;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::config::SimulationConfig;
use crate::lineage::LineageRecord;
use crate::metrics::Metrics;
use crate::world::World;
use std::collections::BTreeMap;
use web_time::Instant;

#[path = "checkpoint.rs"]
mod checkpoint;

use rayon::prelude::*;

use crate::neural_controller::NeuralDecision;
use crate::neural_genome::{ControllerKind, NeuralGenome};
use crate::organism::{Organism, Species};
use crate::profiling::TickProfile;
use crate::sensory_pipeline::SensoryContext;

const FOOD_GRID_CELL_SIZE: f32 = 50.0;
const FEEDING_GRID_CELL_SIZE: f32 = 15.0;
pub struct Simulation {
    pub config: SimulationConfig,
    pub world: World,

    pub tick: u64,
    pub total_births: u64,
    pub total_deaths: u64,
    pub total_kills: u64,
    pub predation_energy: f64,

    lineage: BTreeMap<u64, LineageRecord>,
    rng: ChaCha8Rng,
    food_grid: SpatialGrid,
    feeding_grid: SpatialGrid,
    backend: ComputeBackend,
    organism_grid: SpatialGrid,
    neural_actions: Vec<Option<NeuralDecision>>,
    #[cfg(not(target_arch = "wasm32"))]
    gpu: Option<crate::gpu::GpuInference>,
}

impl Simulation {
    pub fn step_profiled(&mut self) -> TickProfile {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(gpu) = &self.gpu {
            gpu.invalidate_weights();
        }
        self.step_profiled_inner()
    }

    pub fn run_profiled(&mut self, ticks: u64) -> Vec<TickProfile> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(gpu) = &self.gpu {
            gpu.invalidate_weights();
        }
        (0..ticks).map(|_| self.step_profiled_inner()).collect()
    }

    fn step_profiled_inner(&mut self) -> TickProfile {
        let mut profile = TickProfile::default();

        self.tick += 1;

        let start = Instant::now();
        self.spawn_food();
        profile.spawning = start.elapsed();

        let start = Instant::now();
        self.rebuild_grids();
        profile.grid = start.elapsed();

        let start = Instant::now();
        self.update_organisms();
        profile.movement = start.elapsed();

        let start = Instant::now();
        self.feed_organisms();
        self.resolve_predation();
        profile.feeding = start.elapsed();

        let start = Instant::now();
        self.reproduce_organisms();
        profile.reproduction = start.elapsed();

        let start = Instant::now();
        self.remove_dead();
        profile.cleanup = start.elapsed();

        profile
    }

    fn move_organisms_parallel(&mut self) {
        let food = &self.world.food;
        let config = &self.config;

        // parallel sensing
        let targets = sense_food_parallel(&self.world.organisms, food, &self.food_grid, config);

        // preserve rng order
        let mut actions = Vec::with_capacity(self.world.organisms.len());

        for (organism, target) in self.world.organisms.iter().zip(targets) {
            if !organism.is_alive(config) {
                actions.push(MovementAction::Inactive);

                continue;
            }

            let action = match target {
                Some(index) => {
                    let resource = &food[index];

                    MovementAction::SeekFood {
                        x: resource.x,
                        y: resource.y,
                    }
                }

                None => {
                    let turn = self.rng.random_range(-0.3..0.3);

                    MovementAction::Wander { turn }
                }
            };

            actions.push(action);
        }

        // parallel movement and metabolism
        execute_movement_parallel(&mut self.world.organisms, &actions, config);
    }

    pub fn new(config: SimulationConfig) -> Self {
        Self::with_backend(config, ComputeBackend::default())
    }

    pub fn with_backend(config: SimulationConfig, backend: ComputeBackend) -> Self {
        config
            .validate_neural_config()
            .expect("invalid neural configuration");
        let mut rng = ChaCha8Rng::seed_from_u64(config.seed);

        let world = World::new(&config, &mut rng);

        let mut food_grid = SpatialGrid::new(config.width, config.height, FOOD_GRID_CELL_SIZE);
        let mut feeding_grid =
            SpatialGrid::new(config.width, config.height, FEEDING_GRID_CELL_SIZE);
        let organism_grid = SpatialGrid::new(config.width, config.height, FOOD_GRID_CELL_SIZE);

        if backend != ComputeBackend::BruteForce {
            feeding_grid.rebuild_food(&world.food);
        }
        if backend != ComputeBackend::BruteForce {
            food_grid.rebuild_food(&world.food);
        }

        let lineage = world
            .organisms
            .iter()
            .map(|o| (o.id, LineageRecord::new(o, 0)))
            .collect();
        Self {
            lineage,
            config,
            world,

            tick: 0,

            total_births: 0,
            total_deaths: 0,
            total_kills: 0,
            predation_energy: 0.0,

            rng,

            food_grid,
            feeding_grid,
            backend,
            organism_grid,
            neural_actions: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            gpu: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn enable_gpu(&mut self) -> Result<(), String> {
        if self.config.controller == ControllerKind::Heuristic {
            return Err("GPU requires a neural controller".into());
        }
        if self.gpu.is_none() {
            self.gpu = Some(crate::gpu::GpuInference::new()?);
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn disable_gpu(&mut self) {
        self.gpu = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn gpu_enabled(&self) -> bool {
        self.gpu.is_some() && self.config.controller != ControllerKind::Heuristic
    }

    #[cfg(target_arch = "wasm32")]
    pub fn enable_gpu(&mut self) -> Result<(), String> {
        Err("browser GPU is not implemented yet".into())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn disable_gpu(&mut self) {}

    #[cfg(target_arch = "wasm32")]
    pub fn gpu_enabled(&self) -> bool {
        false
    }

    pub fn backend(&self) -> ComputeBackend {
        self.backend
    }

    pub fn active_backend(&self) -> ComputeBackend {
        self.backend.resolve(self.world.organisms.len())
    }

    pub fn set_backend(&mut self, backend: ComputeBackend) {
        self.backend = backend;
    }

    pub fn step(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(gpu) = &self.gpu {
            gpu.invalidate_weights();
        }
        self.step_inner();
    }

    fn step_inner(&mut self) {
        self.tick += 1;

        self.spawn_food();
        self.rebuild_grids();
        self.update_organisms();
        self.feed_organisms();
        self.resolve_predation();
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
        let grid = &self.food_grid;

        let backend = self.backend;

        for organism in &mut self.world.organisms {
            if !organism.is_alive(config) {
                continue;
            }

            let nearest_index = match backend {
                ComputeBackend::BruteForce => {
                    nearest_food_brute_force(organism, food, config.width, config.height)
                }

                ComputeBackend::SerialCpu
                | ComputeBackend::ParallelCpu
                | ComputeBackend::AutoCpu => {
                    nearest_food_spatial(organism, food, grid, config.width, config.height)
                }
            };

            match nearest_index {
                Some(index) => {
                    let resource = &food[index];

                    organism.move_towards(resource.x, resource.y, config);
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
        let grid = &self.feeding_grid;

        let backend = self.backend;

        let mut consumed = vec![false; self.world.food.len()];

        for organism in &mut self.world.organisms {
            if !organism.is_alive(config) {
                continue;
            }

            if config.predation && organism.species == Species::Predator {
                continue;
            }
            if organism.energy >= config.max_energy {
                continue;
            }

            let nearest_index = match backend {
                ComputeBackend::BruteForce => nearest_edible_food_brute_force(
                    organism,
                    &self.world.food,
                    &consumed,
                    config.width,
                    config.height,
                ),

                ComputeBackend::SerialCpu
                | ComputeBackend::ParallelCpu
                | ComputeBackend::AutoCpu => nearest_edible_food_spatial(
                    organism,
                    &self.world.food,
                    &consumed,
                    grid,
                    config.width,
                    config.height,
                ),
            };

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

    fn resolve_predation(&mut self) {
        if !self.config.predation {
            return;
        }
        let config = &self.config;
        let organisms = &mut self.world.organisms;
        self.organism_grid.rebuild_organisms(organisms);
        for i in 0..organisms.len() {
            let predator = &organisms[i];
            if predator.species != Species::Predator || !predator.is_alive(config) {
                continue;
            }
            let aggression = self
                .neural_actions
                .get(i)
                .and_then(|d| *d)
                .map_or(0.0, |d| d.action.aggression);
            if aggression <= 0.5 || predator.energy <= config.attack_cost {
                continue;
            }
            let radius = predator.genome.size + 12.0 + config.attack_range_bonus;
            let mut target = None;
            let mut best = f32::INFINITY;
            self.organism_grid
                .for_each_candidate(predator.x, predator.y, radius, |j| {
                    let prey = &organisms[j];
                    if prey.species != Species::Herbivore || !prey.is_alive(config) {
                        return;
                    }
                    let dx = crate::spatial::wrapped_delta(predator.x, prey.x, config.width);
                    let dy = crate::spatial::wrapped_delta(predator.y, prey.y, config.height);
                    let d = dx * dx + dy * dy;
                    let reach = predator.genome.size + prey.genome.size + config.attack_range_bonus;
                    if d <= reach * reach
                        && (d < best || (d == best && target.is_some_and(|k| j < k)))
                    {
                        best = d;
                        target = Some(j);
                    }
                });
            organisms[i].energy -= config.attack_cost;
            if let Some(j) = target {
                let energy = organisms[j].energy * config.predation_efficiency;
                organisms[j].energy = 0.0;
                let before = organisms[i].energy;
                organisms[i].eat(energy, config);
                self.predation_energy += f64::from(organisms[i].energy - before);
                self.total_kills += 1;
            }
        }
    }

    fn reproduce_organisms(&mut self) {
        let available_space = self
            .config
            .max_population
            .saturating_sub(self.world.organisms.len());

        let mut offspring = Vec::new();
        let mut next_id = self.world.next_organism_id;

        for (index, organism) in self.world.organisms.iter_mut().enumerate() {
            if offspring.len() >= available_space {
                break;
            }

            if self.config.controller != ControllerKind::Heuristic
                && !self
                    .neural_actions
                    .get(index)
                    .and_then(|action| *action)
                    .is_some_and(|decision| decision.action.reproduce)
            {
                continue;
            }

            if let Some(child) = organism.reproduce(next_id, &self.config, &mut self.rng) {
                next_id += 1;
                offspring.push(child);
            }
        }

        for child in &offspring {
            self.lineage
                .insert(child.id, LineageRecord::new(child, self.tick));
        }
        self.total_births += offspring.len() as u64;

        self.world.next_organism_id = next_id;
        self.world.organisms.extend(offspring);
    }

    fn remove_dead(&mut self) {
        let before = self.world.organisms.len();

        let tick = self.tick;
        let config = &self.config;
        let lineage = &mut self.lineage;
        self.world.organisms.retain(|organism| {
            let alive = organism.is_alive(config);
            if !alive && let Some(record) = lineage.get_mut(&organism.id) {
                record.death_tick = Some(tick);
            }
            alive
        });

        let deaths = before - self.world.organisms.len();

        self.total_deaths += deaths as u64;
    }

    pub fn run(&mut self, ticks: u64) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(gpu) = &self.gpu {
            gpu.invalidate_weights();
        }
        for _ in 0..ticks {
            self.step_inner();
        }
    }

    #[cfg(feature = "python")]
    pub(crate) fn run_cached(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.step_inner();
        }
    }

    pub fn metrics(&self) -> Metrics {
        let mut metrics =
            Metrics::collect(&self.world, self.tick, self.total_births, self.total_deaths);
        metrics.total_kills = self.total_kills;
        metrics.predation_energy = self.predation_energy;
        metrics
    }
    fn rebuild_grids(&mut self) {
        if self.backend != ComputeBackend::BruteForce {
            self.food_grid.rebuild_food(&self.world.food);
            self.feeding_grid.rebuild_food(&self.world.food);
        }
    }
    fn update_organisms(&mut self) {
        if self.config.controller != ControllerKind::Heuristic {
            self.move_neural_organisms();
            return;
        }
        match self.active_backend() {
            ComputeBackend::ParallelCpu => self.move_organisms_parallel(),
            _ => self.move_organisms(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn move_neural_gpu(&mut self) {
        let gpu = self.gpu.as_ref().expect("GPU enabled");
        let config = &self.config;
        let organisms = &mut self.world.organisms;
        let sensors = gpu
            .sense(organisms, &self.world.food, config)
            .expect("GPU sensing failed");
        let brains: Vec<_> = organisms.iter().map(|o| &o.brain).collect();
        let ids: Vec<_> = organisms.iter().map(|o| o.id).collect();
        let outputs = gpu
            .infer_resident(config.controller, &brains, &sensors, &ids)
            .expect("GPU inference failed");
        self.neural_actions = organisms
            .iter()
            .zip(outputs)
            .map(|(o, y)| {
                if !o.is_alive(config) {
                    return None;
                }
                Some(NeuralDecision {
                    action: crate::NeuralAction {
                        turn: y[0] * std::f32::consts::PI,
                        speed: (y[1] + 1.0) * 0.5,
                        reproduce: y[2] > 0.0,
                        aggression: if config.controller == ControllerKind::Small {
                            0.0
                        } else {
                            (y[3] + 1.0) * 0.5
                        },
                    },
                    memory: if config.controller == ControllerKind::Recurrent {
                        Some(std::array::from_fn(|i| y[i + 4]))
                    } else {
                        None
                    },
                })
            })
            .collect();
        organisms
            .par_iter_mut()
            .zip(&self.neural_actions)
            .for_each(|(o, decision)| {
                if let Some(d) = decision {
                    o.record_energy();
                    if let (NeuralGenome::Recurrent(n), Some(memory)) = (&mut o.brain, d.memory) {
                        n.memory = memory;
                    }
                    o.apply_neural_action(d.action);
                    o.update(config);
                }
            });
    }

    fn move_neural_organisms(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.gpu_enabled() {
            self.move_neural_gpu();
            return;
        }
        let backend = self.active_backend();

        self.organism_grid.rebuild_organisms(&self.world.organisms);

        let context = SensoryContext {
            organisms: &self.world.organisms,
            food: &self.world.food,
            organism_grid: &self.organism_grid,
            food_grid: &self.food_grid,
            config: &self.config,
        };

        let decide = |organism: &Organism| {
            if !organism.is_alive(&self.config) {
                return None;
            }

            let sensors = if backend == ComputeBackend::BruteForce {
                context.read_brute_force(organism)
            } else {
                context.read(organism)
            };

            NeuralDecision::decide(&organism.brain, &sensors)
        };

        self.neural_actions = if backend == ComputeBackend::ParallelCpu {
            self.world.organisms.par_iter().map(decide).collect()
        } else {
            self.world.organisms.iter().map(decide).collect()
        };

        let config = &self.config;

        let execute = |organism: &mut Organism, action: &Option<NeuralDecision>| {
            if let Some(action) = action {
                organism.record_energy();
                if let (NeuralGenome::Recurrent(network), Some(memory)) =
                    (&mut organism.brain, action.memory)
                {
                    network.memory = memory;
                }
                organism.apply_neural_action(action.action);
                organism.update(config);
            }
        };

        if backend == ComputeBackend::ParallelCpu {
            self.world
                .organisms
                .par_iter_mut()
                .zip(self.neural_actions.par_iter())
                .for_each(|(organism, action)| {
                    execute(organism, action);
                });
        } else {
            self.world
                .organisms
                .iter_mut()
                .zip(self.neural_actions.iter())
                .for_each(|(organism, action)| {
                    execute(organism, action);
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predation_transfers_energy_once() {
        let config = SimulationConfig {
            predation: true,
            controller: ControllerKind::Deep,
            initial_population: 2,
            ..Default::default()
        };
        let mut sim = Simulation::new(config);
        sim.world.organisms[0].species = Species::Predator;
        sim.world.organisms[1].species = Species::Herbivore;
        for o in &mut sim.world.organisms {
            o.x = 10.0;
            o.y = 10.0;
            o.energy = 100.0;
        }
        sim.neural_actions = vec![
            Some(NeuralDecision {
                action: crate::NeuralAction {
                    turn: 0.0,
                    speed: 0.0,
                    reproduce: false,
                    aggression: 1.0,
                },
                memory: None,
            }),
            None,
        ];
        sim.resolve_predation();
        assert_eq!(sim.total_kills, 1);
        assert_eq!(sim.world.organisms[1].energy, 0.0);
        assert_eq!(sim.world.organisms[0].energy, 169.0);
        sim.resolve_predation();
        assert_eq!(sim.total_kills, 1);
        sim.remove_dead();
        assert_eq!(sim.total_deaths, 1);
    }

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
