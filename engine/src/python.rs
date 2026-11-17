use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::{
    ComputeBackend, DenseLayer, NeuralGenome, SensoryContext, Simulation, SimulationConfig,
    SpatialGrid,
};
use serde_json::{Value, json};

#[pyclass(name = "Simulation")]
pub struct PySimulation {
    inner: Simulation,
}

#[pymethods]
impl PySimulation {
    #[new]
    #[pyo3(signature = (config_json=None, backend="auto_cpu"))]
    fn new(config_json: Option<&str>, backend: &str) -> PyResult<Self> {
        let config: SimulationConfig = match config_json {
            Some(json) => {
                serde_json::from_str(json).map_err(|e| PyValueError::new_err(e.to_string()))?
            }
            None => SimulationConfig::default(),
        };

        validate_config(&config)?;

        let mut inner = Simulation::with_backend(config, ComputeBackend::default());
        select_backend(&mut inner, backend)?;
        Ok(Self { inner })
    }

    fn set_backend(&mut self, backend: &str) -> PyResult<()> {
        select_backend(&mut self.inner, backend)
    }

    fn get_backend(&self) -> &str {
        if self.inner.gpu_enabled() {
            "gpu"
        } else {
            self.inner.backend().as_str()
        }
    }

    fn get_active_backend(&self) -> &str {
        if self.inner.gpu_enabled() {
            "gpu"
        } else {
            self.inner.active_backend().as_str()
        }
    }

    fn step(&mut self, py: Python<'_>) {
        py.detach(|| {
            self.inner.run_cached(1);
        });
    }

    fn run(&mut self, py: Python<'_>, ticks: u64) {
        py.detach(|| {
            self.inner.run_cached(ticks);
        });
    }

    fn get_metrics(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.metrics())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn get_state(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.world).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn get_brain(&self, id: u64) -> PyResult<String> {
        let sim = &self.inner;
        let Some(organism) = sim.world.organisms.iter().find(|o| o.id == id) else {
            return Ok("null".to_owned());
        };
        let grid = SpatialGrid::new(sim.config.width, sim.config.height, 50.0);
        let context = SensoryContext {
            organisms: &sim.world.organisms,
            food: &sim.world.food,
            organism_grid: &grid,
            food_grid: &grid,
            config: &sim.config,
        };
        let sensors = context.read_brute_force(organism);
        let (inputs, layers, memory) = match &organism.brain {
            NeuralGenome::Heuristic => (Vec::new(), Vec::new(), Value::Null),
            NeuralGenome::Small(net) => {
                let x = sensors.small_inputs();
                let h = net.hidden.forward(&x).map(f32::tanh);
                let y = net.output.forward(&h).map(f32::tanh);
                (
                    x.to_vec(),
                    vec![
                        layer("Hidden", &net.hidden, &x, &h),
                        layer("Output", &net.output, &h, &y),
                    ],
                    Value::Null,
                )
            }
            NeuralGenome::Deep(net) => {
                let x = sensors.deep_inputs();
                let h1 = net.hidden_1.forward(&x).map(f32::tanh);
                let h2 = net.hidden_2.forward(&h1).map(f32::tanh);
                let y = net.output.forward(&h2).map(f32::tanh);
                (
                    x.to_vec(),
                    vec![
                        layer("Hidden 1", &net.hidden_1, &x, &h1),
                        layer("Hidden 2", &net.hidden_2, &h1, &h2),
                        layer("Output", &net.output, &h2, &y),
                    ],
                    Value::Null,
                )
            }
            NeuralGenome::Recurrent(net) => {
                let x = sensors.deep_inputs();
                let mut combined = [0.0; 32];
                combined[..16].copy_from_slice(&x);
                combined[16..].copy_from_slice(&net.memory);
                let z = net
                    .update
                    .forward(&combined)
                    .map(|v| 1.0 / (1.0 + (-v).exp()));
                let r = net
                    .reset
                    .forward(&combined)
                    .map(|v| 1.0 / (1.0 + (-v).exp()));
                let mut gated = combined;
                for (v, gate) in gated[16..].iter_mut().zip(r) {
                    *v *= gate;
                }
                let c = net.candidate.forward(&gated).map(f32::tanh);
                let (y, next) = net.forward(&x);
                (
                    x.to_vec(),
                    vec![
                        layer("Update gate", &net.update, &combined, &z),
                        layer("Reset gate", &net.reset, &combined, &r),
                        layer("Candidate", &net.candidate, &gated, &c),
                        layer("Output", &net.output, &next, &y),
                    ],
                    json!({"current": net.memory, "next": next}),
                )
            }
        };
        let action = crate::NeuralAction::decide(&organism.brain, &sensors).map(|a| json!({"turn": a.turn, "speed": a.speed, "reproduce": a.reproduce, "aggression": a.aggression}));
        serde_json::to_string(&json!({"id": id, "tick": sim.tick, "generation": organism.generation, "controller": organism.brain.kind(), "parameters": organism.brain.parameter_count(), "inputs": inputs, "layers": layers, "memory": memory, "action": action}))
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn get_lineage(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.lineage().values().collect::<Vec<_>>())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn checkpoint(&self) -> PyResult<String> {
        self.inner.checkpoint().map_err(PyValueError::new_err)
    }

    fn restore_checkpoint(&mut self, checkpoint: &str) -> PyResult<()> {
        let next = Simulation::from_checkpoint(checkpoint).map_err(PyValueError::new_err)?;
        self.inner = next;
        Ok(())
    }

    fn get_config(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner.config).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn reset(&mut self) -> PyResult<()> {
        let config = self.inner.config.clone();
        let backend = self.get_backend();
        let mut next = Simulation::with_backend(config, ComputeBackend::default());
        select_backend(&mut next, backend)?;
        self.inner = next;
        Ok(())
    }

    #[getter]
    fn tick(&self) -> u64 {
        self.inner.tick
    }

    #[getter]
    fn population(&self) -> usize {
        self.inner.world.organisms.len()
    }

    #[getter]
    fn food_count(&self) -> usize {
        self.inner.world.food.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "Simulation(tick={}, population={}, food={})",
            self.inner.tick,
            self.inner.world.organisms.len(),
            self.inner.world.food.len(),
        )
    }
}

fn layer<const IN: usize, const OUT: usize>(
    name: &str,
    layer: &DenseLayer<IN, OUT>,
    inputs: &[f32; IN],
    outputs: &[f32; OUT],
) -> Value {
    json!({"name": name, "weights": layer.weights.iter().map(|row| row.to_vec()).collect::<Vec<_>>(), "biases": layer.biases.to_vec(), "inputs": inputs.to_vec(), "outputs": outputs.to_vec()})
}

fn validate_config(config: &SimulationConfig) -> PyResult<()> {
    if !config.width.is_finite()
        || !config.height.is_finite()
        || config.width <= 0.0
        || config.height <= 0.0
    {
        return Err(PyValueError::new_err(
            "World dimensions must be finite and positive",
        ));
    }

    if config.initial_population > config.max_population {
        return Err(PyValueError::new_err(
            "Initial population exceeds maximum population",
        ));
    }

    if config.initial_food > config.max_food {
        return Err(PyValueError::new_err("Initial food exceeds maximum food"));
    }

    if !config.mutation_rate.is_finite() || !(0.0..=1.0).contains(&config.mutation_rate) {
        return Err(PyValueError::new_err(
            "Mutation rate must be between 0 and 1",
        ));
    }

    if !config.mutation_strength.is_finite() || !(0.0..=1.0).contains(&config.mutation_strength) {
        return Err(PyValueError::new_err(
            "Mutation strength must be between 0 and 1",
        ));
    }

    if !config.initial_energy.is_finite()
        || !config.max_energy.is_finite()
        || config.initial_energy <= 0.0
        || config.initial_energy > config.max_energy
    {
        return Err(PyValueError::new_err("Invalid energy configuration"));
    }

    if !config.food_energy.is_finite() || config.food_energy <= 0.0 || config.max_age == 0 {
        return Err(PyValueError::new_err("Invalid food energy or maximum age"));
    }

    config
        .validate_neural_config()
        .map_err(PyValueError::new_err)?;

    Ok(())
}

fn parse_backend(name: &str) -> PyResult<ComputeBackend> {
    ComputeBackend::from_name(name)
        .ok_or_else(|| PyValueError::new_err(format!("unknown compute backend: {name}")))
}

#[pyfunction]
fn default_config() -> PyResult<String> {
    serde_json::to_string(&SimulationConfig::default())
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn apeiron_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySimulation>()?;
    m.add_function(wrap_pyfunction!(default_config, m)?)?;

    Ok(())
}

fn select_backend(sim: &mut Simulation, backend: &str) -> PyResult<()> {
    if backend == "gpu" {
        sim.enable_gpu().map_err(PyValueError::new_err)?;
        sim.set_backend(ComputeBackend::AutoCpu);
    } else {
        let cpu = parse_backend(backend)?;
        sim.disable_gpu();
        sim.set_backend(cpu);
    }
    Ok(())
}
