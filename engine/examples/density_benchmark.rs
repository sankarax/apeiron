use std::hint::black_box;
use std::time::Instant;

use apeiron_engine::{ComputeBackend, Simulation, SimulationConfig};

struct BenchmarkCase {
    population: usize,
    food: usize,
    ticks: u64,
}

struct BenchmarkResult {
    ms_per_tick: f64,
    tps: f64,
    final_population: usize,
    final_food: usize,
}

fn create_config(case: &BenchmarkCase) -> SimulationConfig {
    let scale = (case.population as f32 / 1000.0).sqrt();

    SimulationConfig {
        width: 1600.0 * scale,
        height: 900.0 * scale,
        initial_population: case.population,
        max_population: case.population,
        initial_food: case.food,
        max_food: case.food,
        food_spawn_rate: case.food,
        max_age: 1_000_000,
        seed: 42,
        ..SimulationConfig::default()
    }
}

fn benchmark(case: &BenchmarkCase, backend: ComputeBackend) -> BenchmarkResult {
    let mut sim = Simulation::with_backend(create_config(case), backend);

    sim.run(5);

    let start = Instant::now();

    for _ in 0..case.ticks {
        sim.step();
    }

    let elapsed = start.elapsed().as_secs_f64();

    black_box(&sim.world);

    BenchmarkResult {
        ms_per_tick: elapsed * 1000.0 / case.ticks as f64,
        tps: case.ticks as f64 / elapsed,
        final_population: sim.world.organisms.len(),
        final_food: sim.world.food.len(),
    }
}

fn print_result(name: &str, result: &BenchmarkResult) {
    println!(
        "{name:>14} {:>12.4} {:>12.2} {:>10.2}x {:>10} {:>10}",
        result.ms_per_tick,
        result.tps,
        result.tps / 60.0,
        result.final_population,
        result.final_food,
    );
}

fn main() {
    println!("APEIRON - CONSTANT DENSITY BENCHMARK");
    println!("rayon threads: {}", rayon::current_num_threads());
    println!("base world: 1600 x 900");
    println!("base population: 1000");
    println!("food to organism ratio: 1:1");

    let cases = [
        BenchmarkCase {
            population: 1000,
            food: 1000,
            ticks: 200,
        },
        BenchmarkCase {
            population: 5000,
            food: 5000,
            ticks: 100,
        },
        BenchmarkCase {
            population: 10000,
            food: 10000,
            ticks: 100,
        },
        BenchmarkCase {
            population: 20000,
            food: 20000,
            ticks: 100,
        },
        BenchmarkCase {
            population: 50000,
            food: 50000,
            ticks: 50,
        },
        BenchmarkCase {
            population: 100000,
            food: 100000,
            ticks: 50,
        },
    ];

    for case in &cases {
        let config = create_config(case);

        println!("world: {:.0} x {:.0}", config.width, config.height);
        println!(
            "\npopulation: {} | food: {} | ticks: {}",
            case.population, case.food, case.ticks
        );

        println!(
            "{:>14} {:>12} {:>12} {:>11} {:>10} {:>10}",
            "backend", "ms/tick", "tps", "real-time", "population", "food"
        );

        println!("{}", "-".repeat(76));

        let mut serial_results = Vec::new();
        let mut parallel_results = Vec::new();
        let mut auto_results = Vec::new();

        // rotate backend order between trials
        for trial in 0..3 {
            match trial {
                0 => {
                    serial_results.push(benchmark(case, ComputeBackend::SerialCpu));
                    parallel_results.push(benchmark(case, ComputeBackend::ParallelCpu));
                    auto_results.push(benchmark(case, ComputeBackend::AutoCpu));
                }
                1 => {
                    parallel_results.push(benchmark(case, ComputeBackend::ParallelCpu));
                    auto_results.push(benchmark(case, ComputeBackend::AutoCpu));
                    serial_results.push(benchmark(case, ComputeBackend::SerialCpu));
                }
                _ => {
                    auto_results.push(benchmark(case, ComputeBackend::AutoCpu));
                    serial_results.push(benchmark(case, ComputeBackend::SerialCpu));
                    parallel_results.push(benchmark(case, ComputeBackend::ParallelCpu));
                }
            }
        }

        serial_results.sort_by(|a, b| a.ms_per_tick.total_cmp(&b.ms_per_tick));
        parallel_results.sort_by(|a, b| a.ms_per_tick.total_cmp(&b.ms_per_tick));
        auto_results.sort_by(|a, b| a.ms_per_tick.total_cmp(&b.ms_per_tick));

        let serial = &serial_results[1];
        let parallel = &parallel_results[1];
        let auto = &auto_results[1];

        print_result("serial", serial);
        print_result("parallel", parallel);
        print_result("auto", auto);

        assert_eq!(serial.final_population, parallel.final_population);
        assert_eq!(serial.final_population, auto.final_population);
        assert_eq!(serial.final_food, parallel.final_food);
        assert_eq!(serial.final_food, auto.final_food);

        println!("parallel speedup: {:.2}x", parallel.tps / serial.tps);
        println!("auto speedup: {:.2}x", auto.tps / serial.tps);
    }
}
