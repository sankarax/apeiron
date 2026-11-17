use apeiron_engine::{ComputeBackend, Simulation, SimulationConfig, TickProfile};

fn profile(population: usize, food: usize, ticks: u64, backend: ComputeBackend) {
    let config = SimulationConfig {
        initial_population: population,
        max_population: population,
        initial_food: food,
        max_food: food,
        food_spawn_rate: food,
        max_age: 1_000_000,
        seed: 42,
        ..SimulationConfig::default()
    };

    let mut sim = Simulation::with_backend(config, backend);

    // warm up before measuring
    sim.run(5);

    let mut profile = TickProfile::default();

    for _ in 0..ticks {
        profile.accumulate(&sim.step_profiled());
    }

    let total = profile.total().as_secs_f64();

    let stages = [
        ("food spawning", profile.spawning),
        ("grid construction", profile.grid),
        ("sensing + movement", profile.movement),
        ("feeding", profile.feeding),
        ("reproduction", profile.reproduction),
        ("cleanup", profile.cleanup),
    ];

    println!("\n{backend:?} | population: {population} | food: {food}");
    println!("{:>22} {:>12} {:>10}", "stage", "ms/tick", "share");
    println!("{}", "-".repeat(48));

    for (name, duration) in stages {
        let seconds = duration.as_secs_f64();
        let ms_per_tick = seconds * 1000.0 / ticks as f64;
        let share = seconds / total * 100.0;

        println!("{name:>22} {ms_per_tick:>12.4} {share:>9.2}%");
    }

    println!("{}", "-".repeat(48));
    println!("{:>22} {:>12.4}", "total", total * 1000.0 / ticks as f64);
}

fn main() {
    println!("APEIRON - PIPELINE PROFILING");
    println!("rayon threads: {}", rayon::current_num_threads());

    let cases = [
        (1_000, 1_000, 200),
        (10_000, 5_000, 100),
        (20_000, 10_000, 50),
        (50_000, 25_000, 20),
        (100_000, 50_000, 10),
    ];

    for (population, food, ticks) in cases {
        profile(population, food, ticks, ComputeBackend::SerialCpu);
        profile(population, food, ticks, ComputeBackend::ParallelCpu);
    }
}
