
use apeiron_engine::{Simulation, SimulationConfig};
use std::time::Instant;

fn main() {
    let config = SimulationConfig::default();

    let mut sim = Simulation::new(config);

    let start = Instant::now();

    println!("APEIRON - Artificial Life Engine");
    println!("================================");
    println!();

    println!(
        "{:>6} {:>6} {:>6} {:>6} {:>6} {:>8} {:>8} {:>8}",
        "Tick",
        "Pop",
        "Food",
        "Births",
        "Deaths",
        "Speed",
        "Energy",
        "Max Gen"
    );

    for _ in 0..20 {
        sim.run(100);

        let m = sim.metrics();

        println!(
            "{:>6} {:>6} {:>6} {:>6} {:>6} {:>8.3} {:>8.2} {:>8}",
            m.tick,
            m.population,
            m.food_count,
            m.total_births,
            m.total_deaths,
            m.average_speed,
            m.average_energy,
            m.max_generation
        );

        if m.population == 0 {
            println!("Population extinct.");
            break;
        }
    }

    let elapsed = start.elapsed();

    println!();
    println!("Elapsed: {:.3?}", elapsed);
    println!(
        "Ticks per second: {:.2}",
        sim.tick as f64 / elapsed.as_secs_f64()
    );

    println!();
    println!("Final evolutionary metrics:");

    println!(
        "{}",
        serde_json::to_string_pretty(&sim.metrics()).unwrap()
    );
}
