use std::time::Instant;

use apeiron_engine::{ComputeBackend, Simulation, SimulationConfig};

fn benchmark(config: &SimulationConfig, backend: ComputeBackend) -> f64 {
    let mut sim = Simulation::with_backend(config.clone(), backend);

    sim.run(20);

    let start = Instant::now();
    sim.run(500);

    start.elapsed().as_secs_f64() * 1000.0 / 500.0
}

fn main() {
    println!("APEIRON - CPU CROSSOVER BENCHMARK");
    println!("rayon threads: {}", rayon::current_num_threads());

    println!(
        "\n{:>10} {:>12} {:>12} {:>10}",
        "population", "serial ms", "parallel ms", "speedup"
    );

    println!("{}", "-".repeat(50));

    for population in [100, 200, 300, 400, 500, 600, 750, 1000, 1500, 2000] {
        let config = SimulationConfig {
            initial_population: population,
            max_population: population,
            initial_food: 1000,
            max_food: 1000,
            food_spawn_rate: 1000,
            max_age: 1_000_000,
            seed: 42,
            ..SimulationConfig::default()
        };

        let mut serial_times = Vec::new();
        let mut parallel_times = Vec::new();

        // alternate execution order to reduce timing bias
        for trial in 0..5 {
            if trial % 2 == 0 {
                serial_times.push(benchmark(&config, ComputeBackend::SerialCpu));
                parallel_times.push(benchmark(&config, ComputeBackend::ParallelCpu));
            } else {
                parallel_times.push(benchmark(&config, ComputeBackend::ParallelCpu));
                serial_times.push(benchmark(&config, ComputeBackend::SerialCpu));
            }
        }

        serial_times.sort_by(f64::total_cmp);
        parallel_times.sort_by(f64::total_cmp);

        let serial = serial_times[2];
        let parallel = parallel_times[2];

        println!(
            "{population:>10} {serial:>12.4} {parallel:>12.4} {:>9.2}x",
            serial / parallel
        );
    }
}
