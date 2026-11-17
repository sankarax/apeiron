use std::hint::black_box;
use std::time::Instant;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use apeiron_engine::{ControllerKind, Organism, SimulationConfig};

const POPULATIONS: [usize; 5] = [100, 1000, 5000, 10000, 20000];

const WARMUP: usize = 3;
const ITERATIONS: usize = 10;
const TRIALS: usize = 5;

fn reproduce_population(
    parents: &mut [Organism],
    config: &SimulationConfig,
    rng: &mut ChaCha8Rng,
) -> Vec<Organism> {
    let mut children = Vec::with_capacity(parents.len());

    let first_child_id = parents.len() as u64;

    for (i, parent) in parents.iter_mut().enumerate() {
        parent.energy = config.max_energy;
        parent.age = 100;

        let child = parent
            .reproduce(first_child_id + i as u64, config, rng)
            .expect("expected offspring");

        children.push(child);
    }

    children
}

fn benchmark(kind: ControllerKind, population: usize, mutation_rate: f32) -> f64 {
    let config = SimulationConfig {
        controller: kind,
        neural_mutation_rate: mutation_rate,
        ..SimulationConfig::default()
    };

    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let mut parents: Vec<_> = (0..population)
        .map(|id| Organism::new(id as u64, &config, &mut rng))
        .collect();

    for _ in 0..WARMUP {
        let children = reproduce_population(&mut parents, &config, &mut rng);

        black_box(children);
    }

    let mut samples = Vec::with_capacity(TRIALS);

    for _ in 0..TRIALS {
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let children = reproduce_population(&mut parents, &config, &mut rng);

            black_box(children);
        }

        let elapsed = start.elapsed().as_secs_f64();

        samples.push(elapsed * 1000.0 / ITERATIONS as f64);
    }

    samples.sort_by(f64::total_cmp);
    samples[TRIALS / 2]
}

fn main() {
    println!("Apeiron organism reproduction benchmark");
    println!("Warmup: {WARMUP}");
    println!("Iterations: {ITERATIONS}");
    println!("Trials: {TRIALS}");

    for kind in [
        ControllerKind::Heuristic,
        ControllerKind::Small,
        ControllerKind::Deep,
    ] {
        println!("\n{kind:?} controller");

        println!(
            "{:>10} {:>14} {:>14} {:>12} {:>16}",
            "Population", "Clone ms", "Mutation ms", "Overhead", "Offspring/s",
        );

        println!("{}", "-".repeat(74));

        for population in POPULATIONS {
            let clone_ms = benchmark(kind, population, 0.0);
            let mutation_ms = benchmark(kind, population, 0.05);

            let overhead = mutation_ms / clone_ms;
            let rate = population as f64 * 1000.0 / mutation_ms;

            println!(
                "{:>10} {:>14.4} {:>14.4} {:>11.2}x {:>16.0}",
                population, clone_ms, mutation_ms, overhead, rate,
            );
        }
    }
}
