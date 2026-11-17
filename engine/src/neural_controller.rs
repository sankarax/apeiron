use crate::neural_genome::NeuralGenome;
use crate::sensors::SensorReading;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuralAction {
    pub turn: f32,
    pub speed: f32,
    pub reproduce: bool,
    pub aggression: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuralDecision {
    pub action: NeuralAction,
    pub memory: Option<crate::recurrent::HiddenState>,
}

impl NeuralAction {
    pub fn decide(brain: &NeuralGenome, sensors: &SensorReading) -> Option<Self> {
        NeuralDecision::decide(brain, sensors).map(|decision| decision.action)
    }
}

impl NeuralDecision {
    pub fn decide(brain: &NeuralGenome, sensors: &SensorReading) -> Option<Self> {
        let (outputs, memory) = match brain {
            NeuralGenome::Heuristic => return None,
            NeuralGenome::Small(network) => {
                let outputs = network.forward(&sensors.small_inputs());
                ([outputs[0], outputs[1], outputs[2], -1.0], None)
            }
            NeuralGenome::Deep(network) => (network.forward(&sensors.deep_inputs()), None),
            NeuralGenome::Recurrent(network) => {
                let (outputs, memory) = network.forward(&sensors.deep_inputs());
                (outputs, Some(memory))
            }
        };
        Some(Self {
            action: NeuralAction {
                turn: outputs[0] * std::f32::consts::PI,
                speed: (outputs[1] + 1.0) * 0.5,
                reproduce: outputs[2] > 0.0,
                aggression: (outputs[3] + 1.0) * 0.5,
            },
            memory,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural_genome::ControllerKind;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn heuristic_has_no_neural_action() {
        let brain = NeuralGenome::Heuristic;
        let sensors = SensorReading::default();

        assert!(NeuralAction::decide(&brain, &sensors).is_none());
    }

    #[test]
    fn neural_actions_are_bounded() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for kind in [ControllerKind::Small, ControllerKind::Deep] {
            for _ in 0..100 {
                let brain = NeuralGenome::random(kind, &mut rng);
                let sensors = SensorReading::default();

                let action = NeuralAction::decide(&brain, &sensors).unwrap();

                assert!((-std::f32::consts::PI..=std::f32::consts::PI).contains(&action.turn));

                assert!((0.0..=1.0).contains(&action.speed));
                assert!((0.0..=1.0).contains(&action.aggression));
            }
        }
    }

    #[test]
    fn neural_decisions_are_deterministic() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let brain = NeuralGenome::random(ControllerKind::Deep, &mut rng);

        let sensors = SensorReading::default();

        assert_eq!(
            NeuralAction::decide(&brain, &sensors),
            NeuralAction::decide(&brain, &sensors),
        );
    }

    #[test]
    fn small_network_has_no_aggression() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let brain = NeuralGenome::random(ControllerKind::Small, &mut rng);

        let action = NeuralAction::decide(&brain, &SensorReading::default()).unwrap();

        assert_eq!(action.aggression, 0.0);
    }
}
