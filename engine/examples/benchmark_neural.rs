use std::hint::black_box;
use std::time::Instant;

use apeiron_engine::{DeepNetwork, RecurrentNetwork, SmallNetwork};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

const POPULATIONS: [usize; 7] = [100, 1000, 5000, 10000, 20000, 50000, 100000];

const WARMUP: usize = 30;
const ITERATIONS: usize = 100;
const TRIALS: usize = 5;

fn measure(mut operation: impl FnMut()) -> f64 {
    for _ in 0..WARMUP {
        operation();
    }

    let mut timings = Vec::with_capacity(TRIALS);

    for _ in 0..TRIALS {
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            operation();
        }

        timings.push(start.elapsed().as_secs_f64() / ITERATIONS as f64);
    }

    timings.sort_by(f64::total_cmp);
    timings[TRIALS / 2]
}

fn benchmark<const IN: usize, const OUT: usize, N>(
    population: usize,
    create_network: impl Fn(&mut ChaCha8Rng) -> N,
    forward: impl Fn(&N, &[f32; IN]) -> [f32; OUT] + Sync,
    parameter_count: usize,
) where
    N: Sync,
{
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let networks: Vec<N> = (0..population).map(|_| create_network(&mut rng)).collect();

    let inputs: Vec<[f32; IN]> = (0..population)
        .map(|_| std::array::from_fn(|_| rng.random_range(-1.0..1.0)))
        .collect();

    let mut serial_outputs = vec![[0.0; OUT]; population];
    let mut parallel_outputs = vec![[0.0; OUT]; population];

    let serial_time = measure(|| {
        for ((network, input), output) in networks.iter().zip(&inputs).zip(&mut serial_outputs) {
            *output = forward(network, input);
        }

        black_box(&serial_outputs);
    });

    let parallel_time = measure(|| {
        networks
            .par_iter()
            .zip(inputs.par_iter())
            .zip(parallel_outputs.par_iter_mut())
            .for_each(|((network, input), output)| {
                *output = forward(network, input);
            });

        black_box(&parallel_outputs);
    });

    assert_eq!(serial_outputs, parallel_outputs);

    let serial_ms = serial_time * 1000.0;
    let parallel_ms = parallel_time * 1000.0;

    let speedup = serial_time / parallel_time;

    let serial_throughput = population as f64 / serial_time;
    let parallel_throughput = population as f64 / parallel_time;

    let parameter_mb = population as f64 * parameter_count as f64 * 4.0 / 1e6;

    println!(
        "{population:>12}\
         {serial_ms:>14.4}\
         {parallel_ms:>14.4}\
         {speedup:>11.2}x\
         {serial_throughput:>18.0}\
         {parallel_throughput:>18.0}\
         {parameter_mb:>12.2}",
    );
}

fn print_header(name: &str) {
    println!();
    println!("{name}");
    println!("{}", "-".repeat(100));

    println!(
        "{:>12}{:>14}{:>14}{:>12}{:>18}{:>18}{:>12}",
        "Population",
        "Serial ms",
        "Parallel ms",
        "Speedup",
        "Serial org/s",
        "Parallel org/s",
        "Params MB",
    );
}

fn main() {
    println!("Apeiron native Rust neural inference benchmark");
    println!("Rayon threads: {}", rayon::current_num_threads());
    println!("Warmup: {WARMUP}");
    println!("Iterations: {ITERATIONS}");
    println!("Trials: {TRIALS}");
    println!("Precision: FP32");
    println!("Timing: median wall-clock latency");

    print_header("Small MLP: (6, 8, 3)");

    for population in POPULATIONS {
        benchmark::<6, 3, SmallNetwork>(
            population,
            SmallNetwork::random,
            SmallNetwork::forward,
            SmallNetwork::parameter_count(),
        );
    }

    print_header("Deep MLP: (16, 32, 16, 4)");

    for population in POPULATIONS {
        benchmark::<16, 4, DeepNetwork>(
            population,
            DeepNetwork::random,
            DeepNetwork::forward,
            DeepNetwork::parameter_count(),
        );
    }

    println!();
    print_header("GRU: 16 inputs, 16 memory, 4 outputs");
    for population in POPULATIONS {
        benchmark::<16, 20, RecurrentNetwork>(
            population,
            |rng| {
                let mut net = RecurrentNetwork::random(rng);
                net.memory = std::array::from_fn(|_| rng.random_range(-1.0..1.0));
                net
            },
            |net, input| {
                let (output, memory) = net.forward(input);
                std::array::from_fn(|i| if i < 4 { output[i] } else { memory[i - 4] })
            },
            RecurrentNetwork::parameter_count(),
        );
    }
    println!("Benchmark complete");
}
