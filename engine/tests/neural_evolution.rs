use apeiron_engine::{
    ComputeBackend, ControllerKind, NeuralGenome, Organism, Simulation, SimulationConfig,
};

fn neural_config(controller: ControllerKind) -> SimulationConfig {
    SimulationConfig {
        controller,
        initial_population: 100,
        max_population: 500,
        initial_food: 500,
        max_food: 1000,
        food_spawn_rate: 20,
        seed: 42,
        ..SimulationConfig::default()
    }
}

fn enable_reproduction(organism: &mut Organism) {
    match &mut organism.brain {
        NeuralGenome::Heuristic => {}

        NeuralGenome::Small(network) => {
            network.output.weights[2].fill(0.0);
            network.output.biases[2] = 1.0;
        }

        NeuralGenome::Deep(network) => {
            network.output.weights[2].fill(0.0);
            network.output.biases[2] = 1.0;
        }

        NeuralGenome::Recurrent(network) => {
            network.output.weights[2].fill(0.0);
            network.output.biases[2] = 1.0;
        }
    }
}

fn assert_simulations_match(a: &Simulation, b: &Simulation) {
    assert_eq!(a.tick, b.tick);
    assert_eq!(a.total_births, b.total_births);
    assert_eq!(a.total_deaths, b.total_deaths);

    assert_eq!(
        serde_json::to_value(&a.world).unwrap(),
        serde_json::to_value(&b.world).unwrap()
    );

    for (left, right) in a.world.organisms.iter().zip(&b.world.organisms) {
        assert_eq!(left.id, right.id);
        assert_eq!(left.brain, right.brain);
        assert_eq!(left.previous_energy, right.previous_energy);
    }
}

#[test]
fn population_initializes_with_correct_controllers() {
    for kind in [
        ControllerKind::Heuristic,
        ControllerKind::Small,
        ControllerKind::Deep,
    ] {
        let simulation = Simulation::new(neural_config(kind));

        assert_eq!(simulation.world.organisms.len(), 100);

        for organism in &simulation.world.organisms {
            assert_eq!(organism.brain.kind(), kind);
        }
    }
}

#[test]
fn neural_population_initialization_is_deterministic() {
    for kind in [ControllerKind::Small, ControllerKind::Deep] {
        let a = Simulation::new(neural_config(kind));
        let b = Simulation::new(neural_config(kind));

        assert_simulations_match(&a, &b);
    }
}

#[test]
fn organisms_have_independent_brains() {
    let simulation = Simulation::new(neural_config(ControllerKind::Deep));
    let organisms = &simulation.world.organisms;

    assert_ne!(organisms[0].brain, organisms[1].brain);
}

#[test]
fn neural_population_is_reproducible() {
    for kind in [ControllerKind::Small, ControllerKind::Deep] {
        let config = neural_config(kind);

        let mut a = Simulation::new(config.clone());
        let mut b = Simulation::new(config);

        for _ in 0..200 {
            a.step();
            b.step();

            assert_simulations_match(&a, &b);
        }
    }
}

#[test]
fn neural_backends_produce_identical_results() {
    for kind in [ControllerKind::Small, ControllerKind::Deep] {
        let config = neural_config(kind);

        let mut serial = Simulation::with_backend(config.clone(), ComputeBackend::SerialCpu);

        let mut parallel = Simulation::with_backend(config, ComputeBackend::ParallelCpu);

        for _ in 0..200 {
            serial.step();
            parallel.step();

            assert_simulations_match(&serial, &parallel);
        }
    }
}

#[test]
fn neural_backend_switching_preserves_state() {
    let config = neural_config(ControllerKind::Deep);

    let mut reference = Simulation::with_backend(config.clone(), ComputeBackend::SerialCpu);

    let mut switched = Simulation::with_backend(config, ComputeBackend::SerialCpu);

    for backend in [
        ComputeBackend::ParallelCpu,
        ComputeBackend::BruteForce,
        ComputeBackend::AutoCpu,
        ComputeBackend::SerialCpu,
    ] {
        switched.set_backend(backend);

        for _ in 0..25 {
            reference.step();
            switched.step();

            assert_simulations_match(&reference, &switched);
        }
    }
}

#[test]
fn offspring_inherit_brains_in_simulation() {
    let config = SimulationConfig {
        initial_population: 1,
        max_population: 10,
        neural_mutation_rate: 0.0,
        ..neural_config(ControllerKind::Deep)
    };

    let mut simulation = Simulation::new(config);

    let parent = &mut simulation.world.organisms[0];

    parent.energy = simulation.config.max_energy;
    parent.age = 100;

    enable_reproduction(parent);

    let original_brain = parent.brain.clone();
    let parent_id = parent.id;

    simulation.step();

    let child = simulation
        .world
        .organisms
        .iter()
        .find(|organism| organism.parent_id == Some(parent_id))
        .expect("expected offspring");

    assert_eq!(child.brain, original_brain);
    assert_eq!(child.generation, 1);
    assert_eq!(simulation.total_births, 1);
}

#[test]
fn offspring_mutate_brains_in_simulation() {
    let config = SimulationConfig {
        initial_population: 1,
        max_population: 10,
        neural_mutation_rate: 1.0,
        neural_mutation_strength: 0.1,
        ..neural_config(ControllerKind::Deep)
    };

    let mut simulation = Simulation::new(config);

    let parent = &mut simulation.world.organisms[0];

    parent.energy = simulation.config.max_energy;
    parent.age = 100;

    enable_reproduction(parent);

    let original_brain = parent.brain.clone();
    let parent_id = parent.id;

    simulation.step();

    let child = simulation
        .world
        .organisms
        .iter()
        .find(|organism| organism.parent_id == Some(parent_id))
        .expect("expected offspring");

    assert_ne!(child.brain, original_brain);
    assert_eq!(child.brain.kind(), ControllerKind::Deep);

    let parent = simulation
        .world
        .organisms
        .iter()
        .find(|organism| organism.id == parent_id)
        .unwrap();

    assert_eq!(parent.brain, original_brain);
}

#[test]
fn neural_inheritance_survives_multiple_generations() {
    let config = SimulationConfig {
        initial_population: 1,
        max_population: 20,
        neural_mutation_rate: 0.0,
        ..neural_config(ControllerKind::Deep)
    };

    let mut simulation = Simulation::new(config);

    enable_reproduction(&mut simulation.world.organisms[0]);

    let original_brain = simulation.world.organisms[0].brain.clone();

    for generation in 0..5 {
        let parent = simulation
            .world
            .organisms
            .iter_mut()
            .find(|organism| organism.generation == generation)
            .unwrap();

        parent.energy = simulation.config.max_energy;
        parent.age = 100;

        simulation.step();

        assert!(
            simulation
                .world
                .organisms
                .iter()
                .any(|organism| organism.generation == generation + 1)
        );
    }

    for organism in &simulation.world.organisms {
        assert_eq!(organism.brain, original_brain);
    }
}

#[test]
fn neural_settings_do_not_affect_heuristic_simulation() {
    let config = neural_config(ControllerKind::Heuristic);

    let modified = SimulationConfig {
        neural_mutation_rate: 1.0,
        neural_mutation_strength: 1.0,
        ..config.clone()
    };

    let mut a = Simulation::new(config);
    let mut b = Simulation::new(modified);

    for _ in 0..200 {
        a.step();
        b.step();

        assert_simulations_match(&a, &b);
    }
}

#[test]
#[should_panic(expected = "invalid neural configuration")]
fn invalid_neural_configuration_is_rejected() {
    let config = SimulationConfig {
        neural_mutation_rate: 1.5,
        ..SimulationConfig::default()
    };

    Simulation::new(config);
}
