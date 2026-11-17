use apeiron_engine::{ComputeBackend, ControllerKind, NeuralGenome, Simulation, SimulationConfig};
use std::time::Instant;

fn main() {
    for kind in [
        ControllerKind::Small,
        ControllerKind::Deep,
        ControllerKind::Recurrent,
    ] {
        let config = SimulationConfig {
            controller: kind,
            initial_population: 1000,
            max_population: 1000,
            initial_food: 1000,
            ..Default::default()
        };
        let mut cpu = Simulation::with_backend(config.clone(), ComputeBackend::ParallelCpu);
        let mut gpu = Simulation::with_backend(config, ComputeBackend::ParallelCpu);
        gpu.enable_gpu().unwrap();
        cpu.step();
        gpu.step();
        assert_eq!(cpu.world.organisms.len(), gpu.world.organisms.len());
        assert_eq!(cpu.world.food.len(), gpu.world.food.len());
        let mut max_error = 0.0f32;
        for (a, b) in cpu.world.organisms.iter().zip(&gpu.world.organisms) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.age, b.age);
            for (x, y) in [
                (a.x, b.x),
                (a.y, b.y),
                (a.vx, b.vx),
                (a.vy, b.vy),
                (a.energy, b.energy),
            ] {
                max_error = max_error.max((x - y).abs());
            }
            if let (NeuralGenome::Recurrent(a), NeuralGenome::Recurrent(b)) = (&a.brain, &b.brain) {
                for (x, y) in a.memory.iter().zip(b.memory) {
                    assert!((x - y).abs() < 1e-5);
                }
            }
        }
        assert!(max_error < 0.002, "{kind:?}: {max_error}");
        println!("{kind:?}: one tick max state error {max_error:e}");
        let start = Instant::now();
        gpu.run(100);
        let elapsed = start.elapsed().as_secs_f64();
        assert_eq!(
            gpu.world.organisms.len() as u64,
            1000 + gpu.total_births - gpu.total_deaths
        );
        println!(
            "{kind:?}: 100 ticks, {:.2} TPS, population {}, accounting passed",
            100.0 / elapsed,
            gpu.world.organisms.len()
        );
    }
}
