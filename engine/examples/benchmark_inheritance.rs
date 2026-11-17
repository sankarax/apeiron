use std::hint::black_box;
use std::time::Instant;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

use apeiron_engine::{ControllerKind, NeuralGenome, NeuralMutation};

const POPULATIONS: [usize; 5] = [100, 1000, 5000, 10000, 20000];

const WARMUP: usize = 3;
const ITERATIONS: usize = 10;
const TRIALS: usize = 5;

fn measure(mut operation: impl FnMut()) -> f64 {
    for _ in 0..WARMUP {
        operation();
    }

    let mut samples = Vec::with_capacity(TRIALS);

    for _ in 0..TRIALS {
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            operation();
        }

        samples.push(start.elapsed().as_secs_f64() * 1000.0 / ITERATIONS as f64);
    }

    samples.sort_by(f64::total_cmp);
    samples[TRIALS / 2]
}

fn benchmark(kind: ControllerKind, population: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mutation = NeuralMutation::default();

    let parents: Vec<_> = (0..population)
        .map(|_| NeuralGenome::random(kind, &mut rng))
        .collect();

    let seeds: Vec<_> = (0..population).map(|i| 1000 + i as u64).collect();

    let initialization_ms = measure(|| {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let genomes: Vec<_> = (0..population)
            .map(|_| NeuralGenome::random(kind, &mut rng))
            .collect();

        black_box(genomes);
    });

    let cloning_ms = measure(|| {
        let children = parents.clone();

        black_box(children);
    });

    let mut genomes = parents.clone();
    let mut mutation_rng = ChaCha8Rng::seed_from_u64(42);

    let mutation_ms = measure(|| {
        for genome in &mut genomes {
            genome.mutate(mutation, &mut mutation_rng);
        }

        black_box(&genomes);
    });

    let serial_ms = measure(|| {
        let children: Vec<_> = parents
            .iter()
            .zip(&seeds)
            .map(|(parent, &seed)| {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                parent.inherit(mutation, &mut rng)
            })
            .collect();

        black_box(children);
    });

    let parallel_ms = measure(|| {
        let children: Vec<_> = parents
            .par_iter()
            .zip(seeds.par_iter())
            .map(|(parent, &seed)| {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                parent.inherit(mutation, &mut rng)
            })
            .collect();

        black_box(children);
    });

    let serial_children: Vec<_> = parents
        .iter()
        .zip(&seeds)
        .map(|(parent, &seed)| {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            parent.inherit(mutation, &mut rng)
        })
        .collect();

    let parallel_children: Vec<_> = parents
        .par_iter()
        .zip(seeds.par_iter())
        .map(|(parent, &seed)| {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            parent.inherit(mutation, &mut rng)
        })
        .collect();

    assert_eq!(serial_children, parallel_children);

    let speedup = serial_ms / parallel_ms;
    let serial_rate = population as f64 * 1000.0 / serial_ms;
    let parallel_rate = population as f64 * 1000.0 / parallel_ms;

    println!(
        "{:>10} {:>12.4} {:>12.4} {:>12.4} {:>12.4} \
         {:>12.4} {:>9.2}x {:>14.0} {:>14.0}",
        population,
        initialization_ms,
        cloning_ms,
        mutation_ms,
        serial_ms,
        parallel_ms,
        speedup,
        serial_rate,
        parallel_rate,
    );
}

fn print_header(kind: ControllerKind) {
    println!("\n{kind:?} controller");

    println!(
        "{:>10} {:>12} {:>12} {:>12} {:>12} \
         {:>12} {:>10} {:>14} {:>14}",
        "Population",
        "Init ms",
        "Clone ms",
        "Mutate ms",
        "Serial ms",
        "Parallel ms",
        "Speedup",
        "Serial/s",
        "Parallel/s",
    );

    println!("{}", "-".repeat(132));
}

fn main() {
    println!("Apeiron neural inheritance benchmark");
    println!("Rayon workers: {}", rayon::current_num_threads());
    println!("Warmup: {WARMUP}");
    println!("Iterations: {ITERATIONS}");
    println!("Trials: {TRIALS}");
    println!("Mutation rate: 0.05");
    println!("Mutation strength: 0.1");

    for kind in [
        ControllerKind::Heuristic,
        ControllerKind::Small,
        ControllerKind::Deep,
    ] {
        print_header(kind);

        for population in POPULATIONS {
            benchmark(kind, population);
        }
    }

    println!("\nSerial and parallel offspring match exactly.");
}
