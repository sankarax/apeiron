use apeiron_engine::{ComputeBackend, ControllerKind, Simulation, SimulationConfig};

#[test]
fn neural_simulations_run() {
    for controller in [ControllerKind::Small, ControllerKind::Deep] {
        let config = SimulationConfig {
            controller,
            ..SimulationConfig::default()
        };

        let mut simulation = Simulation::new(config);

        simulation.run(200);

        assert_eq!(simulation.tick, 200);

        for organism in &simulation.world.organisms {
            assert!(organism.x.is_finite());
            assert!(organism.y.is_finite());
            assert!(organism.energy.is_finite());
        }
    }
}

#[test]
fn neural_backends_remain_equivalent() {
    for controller in [ControllerKind::Small, ControllerKind::Deep] {
        let config = SimulationConfig {
            controller,
            ..SimulationConfig::default()
        };

        let mut brute = Simulation::with_backend(config.clone(), ComputeBackend::BruteForce);

        let mut serial = Simulation::with_backend(config.clone(), ComputeBackend::SerialCpu);

        let mut parallel = Simulation::with_backend(config, ComputeBackend::ParallelCpu);

        for _ in 0..200 {
            brute.step();
            serial.step();
            parallel.step();

            assert_eq!(
                serde_json::to_value(&brute.world).unwrap(),
                serde_json::to_value(&serial.world).unwrap(),
            );

            assert_eq!(
                serde_json::to_value(&serial.world).unwrap(),
                serde_json::to_value(&parallel.world).unwrap(),
            );

            for ((a, b), c) in brute
                .world
                .organisms
                .iter()
                .zip(&serial.world.organisms)
                .zip(&parallel.world.organisms)
            {
                assert_eq!(a.brain, b.brain);
                assert_eq!(b.brain, c.brain);

                assert_eq!(a.previous_energy, b.previous_energy);
                assert_eq!(b.previous_energy, c.previous_energy);
            }
        }
    }
}
