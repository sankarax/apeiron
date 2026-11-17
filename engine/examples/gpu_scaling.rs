use apeiron_engine::{ComputeBackend, ControllerKind, Simulation, SimulationConfig};
use std::time::Instant;

fn main() {
    for kind in [
        ControllerKind::Small,
        ControllerKind::Deep,
        ControllerKind::Recurrent,
    ] {
        for population in [1000, 5000, 10000, 20000] {
            let config = SimulationConfig {
                controller: kind,
                initial_population: population,
                max_population: population,
                initial_food: population / 2,
                max_food: population / 2,
                ..Default::default()
            };
            let mut cpu = Simulation::with_backend(config.clone(), ComputeBackend::ParallelCpu);
            let mut gpu = Simulation::with_backend(config, ComputeBackend::ParallelCpu);
            gpu.enable_gpu().unwrap();
            cpu.run(5);
            gpu.run(5);
            let ticks = 30;
            let start = Instant::now();
            cpu.run(ticks);
            let cpu_ms = start.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
            let start = Instant::now();
            gpu.run(ticks);
            let gpu_ms = start.elapsed().as_secs_f64() * 1000.0 / ticks as f64;
            for sim in [&cpu, &gpu] {
                assert_eq!(
                    sim.world.organisms.len() as u64,
                    population as u64 + sim.total_births - sim.total_deaths
                );
            }
            println!(
                "{kind:?} {population}: CPU {cpu_ms:.3} ms, GPU {gpu_ms:.3} ms, speedup {:.3}x, GPU {:.2} TPS, final pop {}/{}",
                cpu_ms / gpu_ms,
                1000.0 / gpu_ms,
                cpu.world.organisms.len(),
                gpu.world.organisms.len()
            );
        }
    }
}
