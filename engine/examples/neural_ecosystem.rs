use std::time::Instant;

use apeiron_engine::{ComputeBackend, ControllerKind, Simulation, SimulationConfig};

const TICKS: u64 = 10_000;
const REPORT_INTERVAL: u64 = 1000;
const SEEDS: [u64; 3] = [42, 123, 456];

fn run_experiment(controller: ControllerKind, seed: u64) {
    let config = SimulationConfig {
        controller,
        seed,
        initial_population: 500,
        max_population: 5000,
        initial_food: 1000,
        max_food: 3000,
        food_spawn_rate: 20,
        ..SimulationConfig::default()
    };

    let mut simulation = Simulation::with_backend(config, ComputeBackend::AutoCpu);

    println!("\ncontroller: {controller:?} | seed: {seed}");

    println!(
        "{:>6} {:>8} {:>8} {:>8} {:>8} {:>10} {:>8} {:>8}",
        "tick", "pop", "food", "births", "deaths", "energy", "avg gen", "max gen"
    );

    let start = Instant::now();
    let mut peak_population = simulation.world.organisms.len();
    let mut extinction_tick = None;

    for tick in 1..=TICKS {
        simulation.step();

        let population = simulation.world.organisms.len();

        peak_population = peak_population.max(population);

        if tick % REPORT_INTERVAL == 0 || population == 0 {
            let metrics = simulation.metrics();

            println!(
                "{:>6} {:>8} {:>8} {:>8} {:>8} {:>10.2} {:>8.2} {:>8}",
                tick,
                metrics.population,
                metrics.food_count,
                metrics.total_births,
                metrics.total_deaths,
                metrics.average_energy,
                metrics.average_generation,
                metrics.max_generation,
            );
        }

        if population == 0 {
            extinction_tick = Some(tick);
            break;
        }
    }

    let elapsed = start.elapsed().as_secs_f64();

    println!("peak population: {peak_population}");
    println!("extinction tick: {extinction_tick:?}");
    println!("elapsed: {elapsed:.3} s");
    println!("throughput: {:.2} TPS", simulation.tick as f64 / elapsed);
}

fn main() {
    for controller in [
        ControllerKind::Heuristic,
        ControllerKind::Small,
        ControllerKind::Deep,
        ControllerKind::Recurrent,
    ] {
        for seed in SEEDS {
            run_experiment(controller, seed);
        }
    }
}
