use apeiron_engine::{ComputeBackend, Simulation, SimulationConfig};

fn assert_equivalent(config: SimulationConfig, ticks: u64) {
    let seed = config.seed;

    let mut reference = Simulation::with_backend(config.clone(), ComputeBackend::BruteForce);

    let mut optimized = Simulation::with_backend(config, ComputeBackend::SerialCpu);

    for tick in 1..=ticks {
        reference.step();
        optimized.step();

        let reference_state = serde_json::to_string(&reference.world).unwrap();

        let optimized_state = serde_json::to_string(&optimized.world).unwrap();

        assert_eq!(
            reference_state, optimized_state,
            "World mismatch: seed={}, tick={}",
            seed, tick,
        );

        assert_eq!(reference.total_births, optimized.total_births,);

        assert_eq!(reference.total_deaths, optimized.total_deaths,);

        assert_eq!(reference.tick, optimized.tick,);
    }
}

#[test]
fn backends_match_default_ecosystem() {
    assert_equivalent(SimulationConfig::default(), 500);
}

#[test]
fn backends_match_multiple_seeds() {
    for seed in 0..5 {
        let config = SimulationConfig {
            initial_population: 100,
            initial_food: 200,
            seed,
            ..SimulationConfig::default()
        };

        assert_equivalent(config, 200);
    }
}

#[test]
fn backends_match_small_toroidal_world() {
    let config = SimulationConfig {
        width: 103.0,
        height: 77.0,

        initial_population: 100,
        initial_food: 200,

        seed: 123,

        ..SimulationConfig::default()
    };

    assert_equivalent(config, 200);
}

#[test]
fn backends_match_food_scarcity() {
    let config = SimulationConfig {
        initial_population: 200,
        initial_food: 20,
        max_food: 50,
        food_spawn_rate: 1,

        seed: 456,

        ..SimulationConfig::default()
    };

    assert_equivalent(config, 300);
}

#[test]
fn backends_match_high_mutation() {
    let config = SimulationConfig {
        mutation_rate: 0.8,
        mutation_strength: 0.5,

        seed: 789,

        ..SimulationConfig::default()
    };

    assert_equivalent(config, 300);
}

#[test]
fn default_backend_is_serial_cpu() {
    let simulation = Simulation::new(SimulationConfig::default());

    assert_eq!(simulation.backend(), ComputeBackend::AutoCpu,);
}

fn assert_parallel_equivalent(config: SimulationConfig, ticks: u64) {
    let seed = config.seed;

    let mut serial = Simulation::with_backend(config.clone(), ComputeBackend::SerialCpu);

    let mut parallel = Simulation::with_backend(config, ComputeBackend::ParallelCpu);

    for tick in 1..=ticks {
        serial.step();
        parallel.step();

        assert_eq!(
            serde_json::to_string(&serial.world,).unwrap(),
            serde_json::to_string(&parallel.world,).unwrap(),
            "Parallel mismatch: seed={}, tick={}",
            seed,
            tick,
        );

        assert_eq!(serial.total_births, parallel.total_births,);

        assert_eq!(serial.total_deaths, parallel.total_deaths,);

        assert_eq!(serial.tick, parallel.tick,);
    }
}

#[test]
fn parallel_matches_default_ecosystem() {
    assert_parallel_equivalent(SimulationConfig::default(), 500);
}

#[test]
fn parallel_matches_multiple_seeds() {
    for seed in 0..5 {
        let config = SimulationConfig {
            initial_population: 100,
            initial_food: 200,

            seed,

            ..SimulationConfig::default()
        };

        assert_parallel_equivalent(config, 200);
    }
}

#[test]
fn parallel_matches_dense_ecosystem() {
    let config = SimulationConfig {
        initial_population: 5000,
        max_population: 5000,

        initial_food: 2000,
        max_food: 2000,

        food_spawn_rate: 2000,

        seed: 42,

        ..SimulationConfig::default()
    };

    assert_parallel_equivalent(config, 50);
}

#[test]
fn parallel_matches_empty_food_world() {
    let config = SimulationConfig {
        initial_food: 0,
        max_food: 0,
        food_spawn_rate: 0,

        ..SimulationConfig::default()
    };

    assert_parallel_equivalent(config, 200);
}

#[test]
fn profiled_step_matches_normal_step() {
    for backend in [
        ComputeBackend::BruteForce,
        ComputeBackend::SerialCpu,
        ComputeBackend::ParallelCpu,
    ] {
        let config = SimulationConfig::default();

        let mut normal = Simulation::with_backend(config.clone(), backend);
        let mut profiled = Simulation::with_backend(config, backend);

        for _ in 0..200 {
            normal.step();
            profiled.step_profiled();

            assert_eq!(
                serde_json::to_string(&normal.world).unwrap(),
                serde_json::to_string(&profiled.world).unwrap()
            );

            assert_eq!(normal.tick, profiled.tick);
            assert_eq!(normal.total_births, profiled.total_births);
            assert_eq!(normal.total_deaths, profiled.total_deaths);
        }
    }
}

#[test]
fn auto_backend_uses_serial_for_small_populations() {
    let sim = Simulation::new(SimulationConfig {
        initial_population: 200,
        ..SimulationConfig::default()
    });

    assert_eq!(sim.backend(), ComputeBackend::AutoCpu);
    assert_eq!(sim.active_backend(), ComputeBackend::SerialCpu);
}

#[test]
fn auto_backend_uses_parallel_for_large_populations() {
    let sim = Simulation::new(SimulationConfig {
        initial_population: 1000,
        ..SimulationConfig::default()
    });

    assert_eq!(sim.backend(), ComputeBackend::AutoCpu);
    assert_eq!(sim.active_backend(), ComputeBackend::ParallelCpu);
}

#[test]
fn auto_backend_matches_serial() {
    let config = SimulationConfig {
        initial_population: 200,
        initial_food: 2000,
        food_spawn_rate: 100,
        ..SimulationConfig::default()
    };

    let mut serial = Simulation::with_backend(config.clone(), ComputeBackend::SerialCpu);

    let mut auto = Simulation::with_backend(config, ComputeBackend::AutoCpu);

    let mut used_parallel = false;

    for _ in 0..500 {
        used_parallel |= auto.active_backend() == ComputeBackend::ParallelCpu;

        serial.step();
        auto.step();

        assert_eq!(
            serde_json::to_string(&serial.world).unwrap(),
            serde_json::to_string(&auto.world).unwrap()
        );

        assert_eq!(serial.total_births, auto.total_births);
        assert_eq!(serial.total_deaths, auto.total_deaths);
        assert_eq!(serial.tick, auto.tick);
    }

    assert!(used_parallel);
}

#[test]
fn explicit_backends_are_not_overridden() {
    assert_eq!(
        ComputeBackend::SerialCpu.resolve(10_000),
        ComputeBackend::SerialCpu
    );

    assert_eq!(
        ComputeBackend::ParallelCpu.resolve(100),
        ComputeBackend::ParallelCpu
    );

    assert_eq!(
        ComputeBackend::BruteForce.resolve(10_000),
        ComputeBackend::BruteForce
    );
}

#[test]
fn switching_backends_preserves_simulation() {
    let config = SimulationConfig {
        initial_population: 1000,
        initial_food: 1000,
        ..SimulationConfig::default()
    };

    let mut reference = Simulation::with_backend(config.clone(), ComputeBackend::SerialCpu);

    let mut switched = Simulation::with_backend(config, ComputeBackend::SerialCpu);

    let backends = [
        ComputeBackend::ParallelCpu,
        ComputeBackend::AutoCpu,
        ComputeBackend::BruteForce,
        ComputeBackend::SerialCpu,
        ComputeBackend::ParallelCpu,
        ComputeBackend::AutoCpu,
    ];

    for backend in backends {
        switched.set_backend(backend);

        assert_eq!(switched.backend(), backend);

        for _ in 0..20 {
            reference.step();
            switched.step();

            assert_eq!(
                serde_json::to_string(&reference.world).unwrap(),
                serde_json::to_string(&switched.world).unwrap()
            );

            assert_eq!(reference.tick, switched.tick);
            assert_eq!(reference.total_births, switched.total_births);
            assert_eq!(reference.total_deaths, switched.total_deaths);
        }
    }
}
