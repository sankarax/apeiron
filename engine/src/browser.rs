use crate::{ComputeBackend, Simulation, SimulationConfig};
use crate::{DenseLayer, NeuralGenome, SensoryContext, SpatialGrid};
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct BrowserSimulation {
    inner: Simulation,
}

#[wasm_bindgen]
impl BrowserSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<BrowserSimulation, JsValue> {
        let config: SimulationConfig = serde_json::from_str(config_json).map_err(js_error)?;
        config.validate_neural_config().map_err(js_error)?;
        if !(1.0..=4096.0).contains(&config.width)
            || !(1.0..=4096.0).contains(&config.height)
            || config.max_population > 5000
            || config.initial_population > config.max_population
            || config.max_food > 10000
            || config.initial_food > config.max_food
            || config.food_spawn_rate > 10000
            || !(0.0..=1.0).contains(&config.mutation_rate)
            || !(0.0..=1.0).contains(&config.mutation_strength)
        {
            return Err(JsValue::from_str(
                "invalid browser simulation configuration",
            ));
        }
        Ok(Self {
            inner: Simulation::with_backend(config, ComputeBackend::SerialCpu),
        })
    }

    pub fn run(&mut self, ticks: u32) -> Result<(), JsValue> {
        if ticks > 1000 {
            return Err(JsValue::from_str("maximum 1000 ticks per call"));
        }
        self.inner.run(u64::from(ticks));
        Ok(())
    }

    pub fn get_brain(&self, id: u32) -> Result<String, JsValue> {
        let sim = &self.inner;
        let Some(organism) = sim.world.organisms.iter().find(|o| o.id == u64::from(id)) else {
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
            .map_err(js_error)
    }

    pub fn lineage(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner.lineage().values().collect::<Vec<_>>()).map_err(js_error)
    }

    pub fn snapshot(&self) -> Result<String, JsValue> {
        serde_json::to_string(&serde_json::json!({
            "world": self.inner.world,
            "metrics": self.inner.metrics(),
            "config": self.inner.config,
        }))
        .map_err(js_error)
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn layer<const IN: usize, const OUT: usize>(
    name: &str,
    layer: &DenseLayer<IN, OUT>,
    inputs: &[f32; IN],
    outputs: &[f32; OUT],
) -> Value {
    json!({"name": name, "weights": layer.weights.iter().map(|row| row.to_vec()).collect::<Vec<_>>(), "biases": layer.biases.to_vec(), "inputs": inputs.to_vec(), "outputs": outputs.to_vec()})
}
