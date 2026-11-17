use rand::Rng;
use rand_distr::{Distribution, StandardNormal};

pub const PARAMETER_LIMIT: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuralMutation {
    pub rate: f32,
    pub strength: f32,
}

impl Default for NeuralMutation {
    fn default() -> Self {
        Self {
            rate: 0.05,
            strength: 0.1,
        }
    }
}

impl NeuralMutation {
    pub fn is_enabled(&self) -> bool {
        self.rate > 0.0 && self.strength > 0.0
    }
    pub fn new(rate: f32, strength: f32) -> Self {
        assert!(rate.is_finite() && (0.0..=1.0).contains(&rate));
        assert!(strength.is_finite() && strength >= 0.0);

        Self { rate, strength }
    }

    pub fn apply(&self, parameter: &mut f32, rng: &mut impl Rng) {
        if self.rate == 0.0 || self.strength == 0.0 {
            return;
        }

        if rng.random::<f32>() < self.rate {
            let noise: f32 = StandardNormal.sample(rng);

            *parameter =
                (*parameter + noise * self.strength).clamp(-PARAMETER_LIMIT, PARAMETER_LIMIT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn zero_rate_preserves_parameter() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mutation = NeuralMutation::new(0.0, 0.1);
        let mut parameter = 0.5;

        mutation.apply(&mut parameter, &mut rng);

        assert_eq!(parameter, 0.5);
    }

    #[test]
    fn zero_strength_preserves_parameter() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mutation = NeuralMutation::new(1.0, 0.0);
        let mut parameter = 0.5;

        mutation.apply(&mut parameter, &mut rng);

        assert_eq!(parameter, 0.5);
    }

    #[test]
    fn zero_parameter_can_mutate() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mutation = NeuralMutation::new(1.0, 0.1);
        let mut parameter = 0.0;

        mutation.apply(&mut parameter, &mut rng);

        assert_ne!(parameter, 0.0);
    }

    #[test]
    fn mutation_is_deterministic() {
        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);

        let mutation = NeuralMutation::new(0.5, 0.1);

        let mut a = [0.5; 100];
        let mut b = [0.5; 100];

        for parameter in &mut a {
            mutation.apply(parameter, &mut rng_a);
        }

        for parameter in &mut b {
            mutation.apply(parameter, &mut rng_b);
        }

        assert_eq!(a, b);
    }

    #[test]
    fn parameters_remain_bounded() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mutation = NeuralMutation::new(1.0, 100.0);

        let mut parameter = 0.0;

        for _ in 0..1000 {
            mutation.apply(&mut parameter, &mut rng);

            assert!(parameter.is_finite());
            assert!(parameter.abs() <= PARAMETER_LIMIT);
        }
    }

    #[test]
    fn mutation_rate_matches_expectation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mutation = NeuralMutation::new(0.05, 0.1);

        let mut changed = 0;

        for _ in 0..10000 {
            let mut parameter = 0.0;

            mutation.apply(&mut parameter, &mut rng);

            if parameter != 0.0 {
                changed += 1;
            }
        }

        let observed_rate = changed as f32 / 10000.0;

        assert!((observed_rate - 0.05).abs() < 0.01);
    }

    #[test]
    #[should_panic]
    fn invalid_rate_is_rejected() {
        NeuralMutation::new(1.5, 0.1);
    }

    #[test]
    #[should_panic]
    fn invalid_strength_is_rejected() {
        NeuralMutation::new(0.05, -0.1);
    }
}
