use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BrainState {
    brain: NeuralGenome,
    previous_energy: f32,
}

#[derive(Serialize, Deserialize)]
struct Checkpoint {
    version: u32,
    config: SimulationConfig,
    world: World,
    brains: Vec<BrainState>,
    lineage: BTreeMap<u64, LineageRecord>,
    tick: u64,
    total_births: u64,
    total_deaths: u64,
    total_kills: u64,
    predation_energy: f64,
    rng_seed: [u8; 32],
    rng_stream: u64,
    rng_position: String,
    backend: ComputeBackend,
    gpu: bool,
}

impl Simulation {
    pub fn lineage(&self) -> &BTreeMap<u64, LineageRecord> {
        &self.lineage
    }

    pub fn checkpoint(&self) -> Result<String, String> {
        let state = Checkpoint {
            version: 1,
            config: self.config.clone(),
            world: self.world.clone(),
            brains: self
                .world
                .organisms
                .iter()
                .map(|o| BrainState {
                    brain: o.brain.clone(),
                    previous_energy: o.previous_energy,
                })
                .collect(),
            lineage: self.lineage.clone(),
            tick: self.tick,
            total_births: self.total_births,
            total_deaths: self.total_deaths,
            total_kills: self.total_kills,
            predation_energy: self.predation_energy,
            rng_seed: self.rng.get_seed(),
            rng_stream: self.rng.get_stream(),
            rng_position: self.rng.get_word_pos().to_string(),
            backend: self.backend,
            gpu: self.gpu_enabled(),
        };
        serde_json::to_string(&state).map_err(|e| e.to_string())
    }

    pub fn from_checkpoint(json: &str) -> Result<Self, String> {
        let mut state: Checkpoint = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if state.version != 1 {
            return Err("unsupported checkpoint version".into());
        }
        let config = state.config;
        config.validate_neural_config().map_err(str::to_owned)?;
        if !config.width.is_finite()
            || !config.height.is_finite()
            || config.width <= 0.0
            || config.height <= 0.0
            || state.world.width != config.width
            || state.world.height != config.height
            || state.world.organisms.len() > config.max_population
            || state.world.food.len() > config.max_food
            || state.brains.len() != state.world.organisms.len()
        {
            return Err("invalid checkpoint dimensions or population".into());
        }
        for (o, brain) in state.world.organisms.iter_mut().zip(state.brains) {
            if brain.brain.kind() != config.controller {
                return Err("checkpoint controller mismatch".into());
            }
            o.brain = brain.brain;
            o.previous_energy = brain.previous_energy;
        }
        let position: u128 = state
            .rng_position
            .parse()
            .map_err(|_| "invalid rng position")?;
        if position >= (1u128 << 68) {
            return Err("invalid rng position".into());
        }
        let mut rng = ChaCha8Rng::from_seed(state.rng_seed);
        rng.set_stream(state.rng_stream);
        rng.set_word_pos(position);
        let mut sim = Self {
            food_grid: SpatialGrid::new(config.width, config.height, FOOD_GRID_CELL_SIZE),
            feeding_grid: SpatialGrid::new(config.width, config.height, FEEDING_GRID_CELL_SIZE),
            organism_grid: SpatialGrid::new(config.width, config.height, FOOD_GRID_CELL_SIZE),
            config,
            world: state.world,
            lineage: state.lineage,
            tick: state.tick,
            total_births: state.total_births,
            total_deaths: state.total_deaths,
            total_kills: state.total_kills,
            predation_energy: state.predation_energy,
            rng,
            backend: state.backend,
            neural_actions: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            gpu: None,
        };
        sim.rebuild_grids();
        if state.gpu {
            sim.enable_gpu()?;
        }
        Ok(sim)
    }
}
