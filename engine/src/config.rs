use serde::{Deserialize, Serialize};

use crate::neural_genome::ControllerKind;
use crate::neural_mutation::NeuralMutation;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimulationConfig {
    pub predation: bool,
    pub predator_fraction: f32,
    pub predator_metabolism_scale: f32,
    pub attack_cost: f32,
    pub attack_range_bonus: f32,
    pub predation_efficiency: f32,
    pub resource_zones: bool,
    pub hazard_zones: bool,
    pub hazard_damage: f32,
    pub width: f32,
    pub height: f32,
    pub initial_population: usize,
    pub max_population: usize,
    pub initial_food: usize,
    pub max_food: usize,
    pub food_spawn_rate: usize,
    pub food_energy: f32,
    pub initial_energy: f32,
    pub max_energy: f32,
    pub max_age: u32,
    pub mutation_rate: f32,
    pub mutation_strength: f32,
    pub seed: u64,
    pub controller: ControllerKind,
    pub neural_mutation_rate: f32,
    pub neural_mutation_strength: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        let neural_mutation = NeuralMutation::default();

        Self {
            predation: false,
            predator_fraction: 0.2,
            predator_metabolism_scale: 1.0,
            attack_cost: 1.0,
            attack_range_bonus: 2.0,
            predation_efficiency: 0.7,
            resource_zones: false,
            hazard_zones: false,
            hazard_damage: 0.5,
            width: 1600.0,
            height: 900.0,
            initial_population: 200,
            max_population: 5000,
            initial_food: 500,
            max_food: 2000,
            food_spawn_rate: 10,
            food_energy: 25.0,
            initial_energy: 100.0,
            max_energy: 250.0,
            max_age: 3000,
            mutation_rate: 0.1,
            mutation_strength: 0.15,
            seed: 42,
            controller: ControllerKind::Heuristic,
            neural_mutation_rate: neural_mutation.rate,
            neural_mutation_strength: neural_mutation.strength,
        }
    }
}

impl SimulationConfig {
    pub fn neural_mutation(&self) -> NeuralMutation {
        NeuralMutation::new(self.neural_mutation_rate, self.neural_mutation_strength)
    }

    pub fn validate_neural_config(&self) -> Result<(), &'static str> {
        if !self.neural_mutation_rate.is_finite()
            || !(0.0..=1.0).contains(&self.neural_mutation_rate)
        {
            return Err("neural_mutation_rate must be between 0 and 1");
        }

        if !self.neural_mutation_strength.is_finite()
            || !(0.0..=1.0).contains(&self.neural_mutation_strength)
        {
            return Err("neural_mutation_strength must be between 0 and 1");
        }

        if !(0.0..=1.0).contains(&self.predator_fraction) {
            return Err("predator_fraction must be between 0 and 1");
        }
        if self.predation
            && !matches!(
                self.controller,
                ControllerKind::Deep | ControllerKind::Recurrent
            )
        {
            return Err("predation requires deep or recurrent controller");
        }
        for (value, max, name) in [
            (
                self.predator_metabolism_scale,
                2.0,
                "invalid predator_metabolism_scale",
            ),
            (self.attack_cost, 10.0, "invalid attack_cost"),
            (self.attack_range_bonus, 50.0, "invalid attack_range_bonus"),
            (
                self.predation_efficiency,
                1.0,
                "invalid predation_efficiency",
            ),
            (self.hazard_damage, 10.0, "invalid hazard_damage"),
        ] {
            if !(0.0..=max).contains(&value) {
                return Err(name);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_controller_is_heuristic() {
        let config = SimulationConfig::default();

        assert_eq!(config.controller, ControllerKind::Heuristic);
    }

    #[test]
    fn default_neural_mutation_is_correct() {
        let config = SimulationConfig::default();

        assert_eq!(config.neural_mutation(), NeuralMutation::default());
    }

    #[test]
    fn custom_neural_mutation_is_preserved() {
        let config = SimulationConfig {
            neural_mutation_rate: 0.2,
            neural_mutation_strength: 0.3,
            ..SimulationConfig::default()
        };

        assert_eq!(config.neural_mutation(), NeuralMutation::new(0.2, 0.3));
    }

    #[test]
    fn legacy_config_uses_neural_defaults() {
        let mut value = serde_json::to_value(SimulationConfig::default()).unwrap();

        let object = value.as_object_mut().unwrap();

        object.remove("controller");
        object.remove("neural_mutation_rate");
        object.remove("neural_mutation_strength");

        let config: SimulationConfig = serde_json::from_value(value).unwrap();

        assert_eq!(config.controller, ControllerKind::Heuristic);
        assert_eq!(config.neural_mutation(), NeuralMutation::default());
    }

    #[test]
    fn neural_config_round_trip() {
        let config = SimulationConfig {
            controller: ControllerKind::Deep,
            neural_mutation_rate: 0.15,
            neural_mutation_strength: 0.25,
            ..SimulationConfig::default()
        };

        let json = serde_json::to_string(&config).unwrap();

        let restored: SimulationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.controller, ControllerKind::Deep);
        assert_eq!(restored.neural_mutation(), config.neural_mutation());
    }

    #[test]
    fn invalid_controller_is_rejected() {
        let result = serde_json::from_str::<SimulationConfig>(r#"{"controller":"invalid"}"#);

        assert!(result.is_err());
    }

    #[test]
    fn partial_config_preserves_defaults() {
        let config: SimulationConfig =
            serde_json::from_str(r#"{"controller":"small","seed":100}"#).unwrap();

        assert_eq!(config.controller, ControllerKind::Small);
        assert_eq!(config.seed, 100);
        assert_eq!(config.width, SimulationConfig::default().width);
        assert_eq!(config.neural_mutation(), NeuralMutation::default());
    }
}
