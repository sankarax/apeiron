use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Genome {
    pub speed: f32,
    pub sensor_radius: f32,
    pub metabolism: f32,
    pub size: f32,
    pub reproduction_threshold: f32,
    pub hue: f32,
}

impl Genome {
    pub fn random(rng: &mut impl Rng) -> Self {
        Self {
            speed: rng.random_range(0.5..3.0),
            sensor_radius: rng.random_range(20.0..100.0),
            metabolism: rng.random_range(0.1..0.5),
            size: rng.random_range(2.0..6.0),
            reproduction_threshold: rng.random_range(120.0..200.0),
            hue: rng.random_range(0.0..360.0),
        }
    }

    pub fn mutate(&self, rng: &mut impl Rng, rate: f32, strength: f32) -> Self {
        Self {
            speed: mutate_gene(self.speed, rng, rate, strength, 0.2, 6.0),

            sensor_radius: mutate_gene(self.sensor_radius, rng, rate, strength, 10.0, 200.0),

            metabolism: mutate_gene(self.metabolism, rng, rate, strength, 0.05, 1.0),

            size: mutate_gene(self.size, rng, rate, strength, 1.0, 12.0),

            reproduction_threshold: mutate_gene(
                self.reproduction_threshold,
                rng,
                rate,
                strength,
                100.0,
                240.0,
            ),

            hue: if rng.random::<f32>() < rate {
                (self.hue + rng.random_range(-30.0..30.0) * strength).rem_euclid(360.0)
            } else {
                self.hue
            },
        }
    }
}

fn mutate_gene(
    value: f32,
    rng: &mut impl Rng,
    rate: f32,
    strength: f32,
    min: f32,
    max: f32,
) -> f32 {
    if rng.random::<f32>() < rate {
        let change = rng.random_range(-strength..=strength);

        (value * (1.0 + change)).clamp(min, max)
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn random_genome_is_valid() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let genome = Genome::random(&mut rng);

        assert!((0.5..=3.0).contains(&genome.speed));
        assert!((20.0..=100.0).contains(&genome.sensor_radius));
        assert!((0.1..=0.5).contains(&genome.metabolism));
        assert!((2.0..=6.0).contains(&genome.size));
    }

    #[test]
    fn zero_mutation_preserves_genome() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let parent = Genome::random(&mut rng);

        let child = parent.mutate(&mut rng, 0.0, 0.15);

        assert_eq!(parent, child);
    }

    #[test]
    fn mutation_changes_genome() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let parent = Genome::random(&mut rng);

        let child = parent.mutate(&mut rng, 1.0, 0.15);

        assert_ne!(parent, child);
    }

    #[test]
    fn seeded_generation_is_reproducible() {
        let mut rng_a = ChaCha8Rng::seed_from_u64(42);
        let mut rng_b = ChaCha8Rng::seed_from_u64(42);

        assert_eq!(Genome::random(&mut rng_a), Genome::random(&mut rng_b));
    }

    #[test]
    fn mutation_respects_bounds() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut genome = Genome::random(&mut rng);

        for _ in 0..1000 {
            genome = genome.mutate(&mut rng, 1.0, 0.5);

            assert!((0.2..=6.0).contains(&genome.speed));
            assert!((10.0..=200.0).contains(&genome.sensor_radius));
            assert!((0.05..=1.0).contains(&genome.metabolism));
            assert!((1.0..=12.0).contains(&genome.size));
            assert!((100.0..=240.0).contains(&genome.reproduction_threshold));
            assert!((0.0..360.0).contains(&genome.hue));
        }
    }
}
