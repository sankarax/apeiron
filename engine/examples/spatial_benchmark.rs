use apeiron_engine::food_search::{nearest_food_brute_force, nearest_food_spatial};

use apeiron_engine::{Simulation, SimulationConfig, SpatialGrid};

use std::hint::black_box;
use std::time::Instant;

struct BenchmarkCase {
    population: usize,
    food: usize,
    iterations: usize,
}

fn brute_force_search(simulation: &Simulation) -> usize {
    let world = &simulation.world;

    let mut detected = 0;

    for organism in &world.organisms {
        let result = nearest_food_brute_force(organism, &world.food, world.width, world.height);

        if black_box(result).is_some() {
            detected += 1;
        }
    }

    detected
}

fn spatial_search(simulation: &Simulation, grid: &SpatialGrid) -> usize {
    let world = &simulation.world;

    let mut detected = 0;

    for organism in &world.organisms {
        let result = nearest_food_spatial(organism, &world.food, grid, world.width, world.height);

        if black_box(result).is_some() {
            detected += 1;
        }
    }

    detected
}

fn verify_equivalence(simulation: &Simulation, grid: &SpatialGrid) {
    let world = &simulation.world;

    for organism in &world.organisms {
        let reference = nearest_food_brute_force(organism, &world.food, world.width, world.height);

        let optimized =
            nearest_food_spatial(organism, &world.food, grid, world.width, world.height);

        assert_eq!(
            reference, optimized,
            "Mismatch for organism {}",
            organism.id,
        );
    }
}

fn measure(iterations: usize, mut operation: impl FnMut() -> usize) -> (f64, usize) {
    black_box(operation());

    let start = Instant::now();

    let mut detected = 0;

    for _ in 0..iterations {
        detected += black_box(operation());
    }

    let elapsed = start.elapsed().as_secs_f64();

    let ms_per_pass = elapsed * 1000.0 / iterations as f64;

    (ms_per_pass, detected / iterations)
}

fn run_benchmark(case: &BenchmarkCase) {
    let config = SimulationConfig {
        initial_population: case.population,
        max_population: case.population,

        initial_food: case.food,
        max_food: case.food,

        seed: 42,

        ..SimulationConfig::default()
    };

    let simulation = Simulation::new(config);

    println!("\nPopulation: {} | Food: {}", case.population, case.food,);

    println!(
        "{:>14} {:>10} {:>12} {:>12} {:>10}",
        "ALGORITHM", "CELL SIZE", "MS/PASS", "SPEEDUP", "DETECTED",
    );

    println!("{}", "-".repeat(65));

    let (brute_ms, brute_detected) = measure(case.iterations, || {
        brute_force_search(black_box(&simulation))
    });

    println!(
        "{:>14} {:>10} {:>12.3} {:>11.2}x {:>10}",
        "Brute force", "-", brute_ms, 1.0, brute_detected,
    );

    for cell_size in [10.0, 25.0, 50.0, 75.0, 100.0, 200.0] {
        let mut grid = SpatialGrid::new(simulation.world.width, simulation.world.height, cell_size);

        grid.rebuild_food(&simulation.world.food);

        verify_equivalence(&simulation, &grid);

        let iterations = case.iterations.max(100);

        let (spatial_ms, detected) = measure(iterations, || {
            grid.rebuild_food(&simulation.world.food);

            spatial_search(black_box(&simulation), black_box(&grid))
        });

        assert_eq!(brute_detected, detected,);

        let speedup = brute_ms / spatial_ms;

        println!(
            "{:>14} {:>10.0} {:>12.3} {:>11.2}x {:>10}",
            "Spatial grid", cell_size, spatial_ms, speedup, detected,
        );
    }
}

fn main() {
    let cases = [
        BenchmarkCase {
            population: 200,
            food: 500,
            iterations: 100,
        },
        BenchmarkCase {
            population: 1000,
            food: 1000,
            iterations: 50,
        },
        BenchmarkCase {
            population: 5000,
            food: 2000,
            iterations: 20,
        },
        BenchmarkCase {
            population: 10000,
            food: 5000,
            iterations: 10,
        },
        BenchmarkCase {
            population: 20000,
            food: 10000,
            iterations: 5,
        },
    ];

    println!("\nAPEIRON — SPATIAL BENCHMARK");

    println!("Seed: 42");
    println!("World: 1600 x 900");
    println!("Grid rebuild included in spatial timing");

    for case in &cases {
        run_benchmark(case);
    }

    println!("\nBenchmark complete.");
}
