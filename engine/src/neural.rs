use crate::neural_mutation::NeuralMutation;
use rand::Rng;
use serde::{Deserialize, Serialize};

pub const SMALL_INPUTS: usize = 6;
pub const SMALL_HIDDEN: usize = 8;
pub const SMALL_OUTPUTS: usize = 3;

pub const DEEP_INPUTS: usize = 16;
pub const DEEP_HIDDEN_1: usize = 32;
pub const DEEP_HIDDEN_2: usize = 16;
pub const DEEP_OUTPUTS: usize = 4;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "[[f32; IN]; OUT]: Serialize, [f32; OUT]: Serialize",
    deserialize = "[[f32; IN]; OUT]: Deserialize<'de>, [f32; OUT]: Deserialize<'de>"
))]
pub struct DenseLayer<const IN: usize, const OUT: usize> {
    pub weights: [[f32; IN]; OUT],
    pub biases: [f32; OUT],
}

impl<const IN: usize, const OUT: usize> DenseLayer<IN, OUT> {
    pub fn mutate(&mut self, mutation: NeuralMutation, rng: &mut impl Rng) {
        for row in &mut self.weights {
            for weight in row {
                mutation.apply(weight, rng);
            }
        }

        for bias in &mut self.biases {
            mutation.apply(bias, rng);
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        let limit = (6.0 / (IN + OUT) as f32).sqrt();

        Self {
            weights: std::array::from_fn(|_| {
                std::array::from_fn(|_| rng.random_range(-limit..limit))
            }),
            biases: [0.0; OUT],
        }
    }

    pub const fn parameter_count() -> usize {
        IN * OUT + OUT
    }

    pub fn forward(&self, inputs: &[f32; IN]) -> [f32; OUT] {
        std::array::from_fn(|i| {
            self.weights[i]
                .iter()
                .zip(inputs)
                .fold(self.biases[i], |sum, (weight, input)| sum + weight * input)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmallNetwork {
    pub hidden: DenseLayer<SMALL_INPUTS, SMALL_HIDDEN>,
    pub output: DenseLayer<SMALL_HIDDEN, SMALL_OUTPUTS>,
}

impl SmallNetwork {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            hidden: DenseLayer::random(rng),
            output: DenseLayer::random(rng),
        }
    }

    pub const fn parameter_count() -> usize {
        DenseLayer::<SMALL_INPUTS, SMALL_HIDDEN>::parameter_count()
            + DenseLayer::<SMALL_HIDDEN, SMALL_OUTPUTS>::parameter_count()
    }

    pub fn forward(&self, inputs: &[f32; SMALL_INPUTS]) -> [f32; SMALL_OUTPUTS] {
        let hidden = self.hidden.forward(inputs).map(f32::tanh);
        self.output.forward(&hidden).map(f32::tanh)
    }

    pub fn mutate(&mut self, mutation: NeuralMutation, rng: &mut impl Rng) {
        self.hidden.mutate(mutation, rng);
        self.output.mutate(mutation, rng);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeepNetwork {
    pub hidden_1: DenseLayer<DEEP_INPUTS, DEEP_HIDDEN_1>,
    pub hidden_2: DenseLayer<DEEP_HIDDEN_1, DEEP_HIDDEN_2>,
    pub output: DenseLayer<DEEP_HIDDEN_2, DEEP_OUTPUTS>,
}

impl DeepNetwork {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            hidden_1: DenseLayer::random(rng),
            hidden_2: DenseLayer::random(rng),
            output: DenseLayer::random(rng),
        }
    }

    pub const fn parameter_count() -> usize {
        DenseLayer::<DEEP_INPUTS, DEEP_HIDDEN_1>::parameter_count()
            + DenseLayer::<DEEP_HIDDEN_1, DEEP_HIDDEN_2>::parameter_count()
            + DenseLayer::<DEEP_HIDDEN_2, DEEP_OUTPUTS>::parameter_count()
    }

    pub fn forward(&self, inputs: &[f32; DEEP_INPUTS]) -> [f32; DEEP_OUTPUTS] {
        let hidden_1 = self.hidden_1.forward(inputs).map(f32::tanh);
        let hidden_2 = self.hidden_2.forward(&hidden_1).map(f32::tanh);
        self.output.forward(&hidden_2).map(f32::tanh)
    }

    pub fn mutate(&mut self, mutation: NeuralMutation, rng: &mut impl Rng) {
        self.hidden_1.mutate(mutation, rng);
        self.hidden_2.mutate(mutation, rng);
        self.output.mutate(mutation, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn dense_layer_parameter_count() {
        assert_eq!(DenseLayer::<16, 32>::parameter_count(), 544);
    }

    #[test]
    fn small_network_parameter_count() {
        assert_eq!(SmallNetwork::parameter_count(), 83);
    }

    #[test]
    fn deep_network_parameter_count() {
        assert_eq!(DeepNetwork::parameter_count(), 1140);
    }

    #[test]
    fn initialization_is_deterministic() {
        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);

        assert_eq!(
            DeepNetwork::random(&mut rng_a),
            DeepNetwork::random(&mut rng_b)
        );
    }

    #[test]
    fn different_seeds_produce_different_networks() {
        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(43);

        assert_ne!(
            DeepNetwork::random(&mut rng_a),
            DeepNetwork::random(&mut rng_b)
        );
    }

    #[test]
    fn weights_are_within_initialization_limits() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let layer = DenseLayer::<16, 32>::random(&mut rng);
        let limit = (6.0_f32 / 48.0).sqrt();

        for weight in layer.weights.iter().flatten() {
            assert!(weight.abs() <= limit);
        }
    }

    #[test]
    fn biases_start_at_zero() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let layer = DenseLayer::<16, 32>::random(&mut rng);

        assert_eq!(layer.biases, [0.0; 32]);
    }

    #[test]
    fn deep_network_memory_size() {
        assert_eq!(
            std::mem::size_of::<DeepNetwork>(),
            1140 * std::mem::size_of::<f32>()
        );
    }

    #[test]
    fn dense_layer_forward_is_correct() {
        let layer = DenseLayer::<2, 2> {
            weights: [[1.0, 2.0], [-1.0, 3.0]],
            biases: [0.5, -0.5],
        };

        assert_eq!(layer.forward(&[2.0, 3.0]), [8.5, 6.5]);
    }

    #[test]
    fn zero_inputs_produce_zero_outputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let network = DeepNetwork::random(&mut rng);

        assert_eq!(network.forward(&[0.0; DEEP_INPUTS]), [0.0; DEEP_OUTPUTS]);
    }

    #[test]
    fn small_network_outputs_are_bounded() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let network = SmallNetwork::random(&mut rng);

        for value in network.forward(&[1.0; SMALL_INPUTS]) {
            assert!(value.is_finite());
            assert!((-1.0..=1.0).contains(&value));
        }
    }

    #[test]
    fn deep_network_outputs_are_bounded() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let network = DeepNetwork::random(&mut rng);

        for _ in 0..100 {
            let inputs = std::array::from_fn(|_| rng.random_range(-1.0..=1.0));

            for value in network.forward(&inputs) {
                assert!(value.is_finite());
                assert!((-1.0..=1.0).contains(&value));
            }
        }
    }

    #[test]
    fn inference_is_deterministic() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let network = DeepNetwork::random(&mut rng);
        let inputs = [0.5; DEEP_INPUTS];

        assert_eq!(network.forward(&inputs), network.forward(&inputs));
    }

    #[test]
    fn different_inputs_change_outputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let network = DeepNetwork::random(&mut rng);

        assert_ne!(
            network.forward(&[0.5; DEEP_INPUTS]),
            network.forward(&[-0.5; DEEP_INPUTS])
        );
    }

    #[test]
    fn small_network_mutates() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let parent = SmallNetwork::random(&mut rng);
        let mut child = parent.clone();

        child.mutate(NeuralMutation::new(1.0, 0.1), &mut rng);

        assert_ne!(parent, child);
    }

    #[test]
    fn deep_network_mutates() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let parent = DeepNetwork::random(&mut rng);
        let mut child = parent.clone();

        child.mutate(NeuralMutation::new(1.0, 0.1), &mut rng);

        assert_ne!(parent, child);
    }

    #[test]
    fn zero_mutation_preserves_network() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let parent = DeepNetwork::random(&mut rng);
        let mut child = parent.clone();

        child.mutate(NeuralMutation::new(0.0, 0.1), &mut rng);

        assert_eq!(parent, child);
    }

    #[test]
    fn mutated_network_produces_finite_outputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let mut network = DeepNetwork::random(&mut rng);

        for _ in 0..100 {
            network.mutate(NeuralMutation::new(0.1, 0.5), &mut rng);
        }

        let inputs = [0.5; DEEP_INPUTS];
        let outputs = network.forward(&inputs);

        assert!(
            outputs
                .iter()
                .all(|value| { value.is_finite() && value.abs() <= 1.0 })
        );
    }
}
