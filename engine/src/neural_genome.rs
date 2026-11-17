use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::neural::{DeepNetwork, SmallNetwork};
use crate::neural_mutation::NeuralMutation;
use crate::recurrent::{MEMORY_SIZE, RecurrentNetwork};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControllerKind {
    #[default]
    Heuristic,
    Small,
    Deep,
    Recurrent,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum NeuralGenome {
    #[default]
    Heuristic,
    Small(Box<SmallNetwork>),
    Deep(Box<DeepNetwork>),
    Recurrent(Box<RecurrentNetwork>),
}

impl NeuralGenome {
    pub fn random(kind: ControllerKind, rng: &mut impl Rng) -> Self {
        match kind {
            ControllerKind::Heuristic => Self::Heuristic,
            ControllerKind::Small => Self::Small(Box::new(SmallNetwork::random(rng))),
            ControllerKind::Deep => Self::Deep(Box::new(DeepNetwork::random(rng))),
            ControllerKind::Recurrent => Self::Recurrent(Box::new(RecurrentNetwork::random(rng))),
        }
    }

    pub fn kind(&self) -> ControllerKind {
        match self {
            Self::Heuristic => ControllerKind::Heuristic,
            Self::Small(_) => ControllerKind::Small,
            Self::Deep(_) => ControllerKind::Deep,
            Self::Recurrent(_) => ControllerKind::Recurrent,
        }
    }

    pub fn parameter_count(&self) -> usize {
        match self {
            Self::Heuristic => 0,
            Self::Small(_) => SmallNetwork::parameter_count(),
            Self::Deep(_) => DeepNetwork::parameter_count(),
            Self::Recurrent(_) => RecurrentNetwork::parameter_count(),
        }
    }

    pub fn parameter_bytes(&self) -> usize {
        self.parameter_count() * std::mem::size_of::<f32>()
    }

    pub fn is_neural(&self) -> bool {
        !matches!(self, Self::Heuristic)
    }

    pub fn mutate(&mut self, mutation: NeuralMutation, rng: &mut impl Rng) {
        if !mutation.is_enabled() {
            return;
        }

        match self {
            Self::Heuristic => {}
            Self::Small(network) => network.mutate(mutation, rng),
            Self::Deep(network) => network.mutate(mutation, rng),
            Self::Recurrent(network) => network.mutate(mutation, rng),
        }
    }

    pub fn inherit(&self, mutation: NeuralMutation, rng: &mut impl Rng) -> Self {
        let mut child = self.clone();
        child.mutate(mutation, rng);
        if let Self::Recurrent(network) = &mut child {
            network.memory = [0.0; MEMORY_SIZE];
        }
        child
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn default_genome_is_heuristic() {
        let genome = NeuralGenome::default();

        assert_eq!(genome.kind(), ControllerKind::Heuristic);
        assert!(!genome.is_neural());
        assert_eq!(genome.parameter_count(), 0);
    }

    #[test]
    fn small_genome_initializes_correctly() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let genome = NeuralGenome::random(ControllerKind::Small, &mut rng);

        assert_eq!(genome.kind(), ControllerKind::Small);
        assert!(genome.is_neural());
        assert_eq!(genome.parameter_count(), 83);
        assert_eq!(genome.parameter_bytes(), 332);
    }

    #[test]
    fn deep_genome_initializes_correctly() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let genome = NeuralGenome::random(ControllerKind::Deep, &mut rng);

        assert_eq!(genome.kind(), ControllerKind::Deep);
        assert!(genome.is_neural());
        assert_eq!(genome.parameter_count(), 1140);
        assert_eq!(genome.parameter_bytes(), 4560);
    }

    #[test]
    fn genome_initialization_is_deterministic() {
        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);

        let a = NeuralGenome::random(ControllerKind::Deep, &mut rng_a);
        let b = NeuralGenome::random(ControllerKind::Deep, &mut rng_b);

        assert_eq!(a, b);
    }

    #[test]
    fn inheritance_preserves_controller_kind() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for kind in [
            ControllerKind::Heuristic,
            ControllerKind::Small,
            ControllerKind::Deep,
        ] {
            let parent = NeuralGenome::random(kind, &mut rng);
            let child = parent.inherit(NeuralMutation::default(), &mut rng);

            assert_eq!(child.kind(), parent.kind());
        }
    }

    #[test]
    fn zero_mutation_produces_identical_child() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for kind in [ControllerKind::Small, ControllerKind::Deep] {
            let parent = NeuralGenome::random(kind, &mut rng);

            let child = parent.inherit(NeuralMutation::new(0.0, 0.1), &mut rng);

            assert_eq!(parent, child);
        }
    }

    #[test]
    fn inheritance_does_not_modify_parent() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let parent = NeuralGenome::random(ControllerKind::Deep, &mut rng);
        let original = parent.clone();

        let child = parent.inherit(NeuralMutation::new(1.0, 0.1), &mut rng);

        assert_eq!(parent, original);
        assert_ne!(parent, child);
    }

    #[test]
    fn inheritance_is_deterministic() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let parent = NeuralGenome::random(ControllerKind::Deep, &mut rng);

        let mut rng_a = ChaCha8Rng::seed_from_u64(100);
        let mut rng_b = ChaCha8Rng::seed_from_u64(100);

        let mutation = NeuralMutation::default();

        let a = parent.inherit(mutation, &mut rng_a);
        let b = parent.inherit(mutation, &mut rng_b);

        assert_eq!(a, b);
    }

    #[test]
    fn heuristic_genome_does_not_consume_rng() {
        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);

        let parent = NeuralGenome::random(ControllerKind::Heuristic, &mut rng_a);

        let child = parent.inherit(NeuralMutation::default(), &mut rng_a);

        assert_eq!(child, NeuralGenome::Heuristic);
        assert_eq!(rng_a.random::<u64>(), rng_b.random::<u64>());
    }

    #[test]
    fn genome_has_compact_inline_storage() {
        let size = std::mem::size_of::<NeuralGenome>();

        assert!(size <= 32);
    }

    #[test]
    fn controller_kind_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&ControllerKind::Deep).unwrap(),
            "\"deep\""
        );

        let kind: ControllerKind = serde_json::from_str("\"small\"").unwrap();

        assert_eq!(kind, ControllerKind::Small);
    }
}
