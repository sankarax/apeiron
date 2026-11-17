use crate::neural::{DEEP_INPUTS, DEEP_OUTPUTS, DenseLayer};
use crate::neural_mutation::NeuralMutation;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const MEMORY_SIZE: usize = 16;
const GATE_INPUTS: usize = DEEP_INPUTS + MEMORY_SIZE;
pub type HiddenState = [f32; MEMORY_SIZE];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecurrentNetwork {
    pub update: DenseLayer<GATE_INPUTS, MEMORY_SIZE>,
    pub reset: DenseLayer<GATE_INPUTS, MEMORY_SIZE>,
    pub candidate: DenseLayer<GATE_INPUTS, MEMORY_SIZE>,
    pub output: DenseLayer<MEMORY_SIZE, DEEP_OUTPUTS>,
    pub memory: HiddenState,
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

impl RecurrentNetwork {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            update: DenseLayer::random(rng),
            reset: DenseLayer::random(rng),
            candidate: DenseLayer::random(rng),
            output: DenseLayer::random(rng),
            memory: [0.0; MEMORY_SIZE],
        }
    }

    pub const fn parameter_count() -> usize {
        3 * DenseLayer::<GATE_INPUTS, MEMORY_SIZE>::parameter_count()
            + DenseLayer::<MEMORY_SIZE, DEEP_OUTPUTS>::parameter_count()
    }

    pub fn forward(&self, inputs: &[f32; DEEP_INPUTS]) -> ([f32; DEEP_OUTPUTS], HiddenState) {
        let mut combined = [0.0; GATE_INPUTS];
        combined[..DEEP_INPUTS].copy_from_slice(inputs);
        combined[DEEP_INPUTS..].copy_from_slice(&self.memory);
        let update = self.update.forward(&combined).map(sigmoid);
        let reset = self.reset.forward(&combined).map(sigmoid);
        for (value, gate) in combined[DEEP_INPUTS..].iter_mut().zip(reset) {
            *value *= gate;
        }
        let candidate = self.candidate.forward(&combined).map(f32::tanh);
        let memory =
            std::array::from_fn(|i| (1.0 - update[i]) * self.memory[i] + update[i] * candidate[i]);
        (self.output.forward(&memory).map(f32::tanh), memory)
    }

    pub fn mutate(&mut self, mutation: NeuralMutation, rng: &mut impl Rng) {
        self.update.mutate(mutation, rng);
        self.reset.mutate(mutation, rng);
        self.candidate.mutate(mutation, rng);
        self.output.mutate(mutation, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn parameter_count_and_storage() {
        assert_eq!(RecurrentNetwork::parameter_count(), 1652);
        assert_eq!(std::mem::size_of::<RecurrentNetwork>(), 6672);
    }

    #[test]
    fn memory_changes_future_outputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut network = RecurrentNetwork::random(&mut rng);
        let inputs = [0.5; DEEP_INPUTS];
        let (first, memory) = network.forward(&inputs);
        assert_eq!(network.memory, [0.0; MEMORY_SIZE]);
        network.memory = memory;
        let (second, _) = network.forward(&inputs);
        assert_ne!(first, second);
    }

    #[test]
    fn recurrent_values_stay_bounded() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut network = RecurrentNetwork::random(&mut rng);
        for _ in 0..100 {
            let inputs = std::array::from_fn(|_| rng.random_range(-1.0..=1.0));
            let (outputs, memory) = network.forward(&inputs);
            assert!(
                outputs
                    .iter()
                    .chain(&memory)
                    .all(|x| x.is_finite() && x.abs() <= 1.000001)
            );
            network.memory = memory;
        }
    }

    #[test]
    fn zero_weights_retain_half_the_memory() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut network = RecurrentNetwork::random(&mut rng);
        for gate in [
            &mut network.update,
            &mut network.reset,
            &mut network.candidate,
        ] {
            gate.weights = [[0.0; GATE_INPUTS]; MEMORY_SIZE];
            gate.biases = [0.0; MEMORY_SIZE];
        }
        network.memory = [0.8; MEMORY_SIZE];
        let (_, memory) = network.forward(&[0.0; DEEP_INPUTS]);
        assert_eq!(memory, [0.4; MEMORY_SIZE]);
    }
}
