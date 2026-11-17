use apeiron_engine::{ComputeBackend, ControllerKind, Simulation, SimulationConfig};
use std::time::Instant;

fn main() {
    println!("rayon threads: {}", rayon::current_num_threads());
    println!("times are mean milliseconds per tick; movement includes sensing and inference");
    for (kind, predation) in [
        (ControllerKind::Small, false),
        (ControllerKind::Deep, false),
        (ControllerKind::Recurrent, false),
        (ControllerKind::Recurrent, true),
    ] {
        for population in [1000, 5000, 20000] {
            let config = SimulationConfig {
                controller: kind,
                predation,
                predator_metabolism_scale: 0.4,
                initial_population: population,
                max_population: population,
                initial_food: population / 2,
                max_food: population / 2,
                food_spawn_rate: population / 2,
                ..Default::default()
            };
            for gpu in [false, true] {
                let mut sim = Simulation::with_backend(config.clone(), ComputeBackend::ParallelCpu);
                if gpu {
                    sim.enable_gpu().expect("GPU initialization failed");
                }
                sim.run(5);
                let start = Instant::now();
                let profiles = sim.run_profiled(30);
                let elapsed = start.elapsed().as_secs_f64();
                let mut sums = [0.0; 6];
                for p in profiles {
                    for (sum, duration) in sums.iter_mut().zip([
                        p.spawning,
                        p.grid,
                        p.movement,
                        p.feeding,
                        p.reproduction,
                        p.cleanup,
                    ]) {
                        *sum += duration.as_secs_f64() * 1000.0 / 30.0;
                    }
                }
                let m = sim.metrics();
                assert_eq!(
                    m.population as u64,
                    population as u64 + m.total_births - m.total_deaths
                );
                assert!(m.total_kills <= m.total_deaths);
                println!(
                    "{kind:?} predation={predation} initial={population} backend={} final={} kills={} ms={:.4} tps={:.2} spawn={:.4} grid={:.4} movement={:.4} feed={:.4} reproduction={:.4} cleanup={:.4}",
                    if gpu { "gpu" } else { "cpu" },
                    m.population,
                    m.total_kills,
                    elapsed * 1000.0 / 30.0,
                    30.0 / elapsed,
                    sums[0],
                    sums[1],
                    sums[2],
                    sums[3],
                    sums[4],
                    sums[5]
                );
            }
        }
    }
}
