use apeiron_engine::{
    ComputeBackend, ControllerKind, NeuralGenome, Organism, Simulation, SimulationConfig,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn child_inherits_weights_but_not_memory() {
    let config = SimulationConfig {
        controller: ControllerKind::Recurrent,
        neural_mutation_rate: 0.0,
        ..SimulationConfig::default()
    };
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut parent = Organism::new(0, &config, &mut rng);
    parent.age = 100;
    parent.energy = config.max_energy;
    let NeuralGenome::Recurrent(network) = &mut parent.brain else {
        panic!("wrong controller")
    };
    network.memory = [0.5; 16];
    let mut expected = parent.brain.clone();
    if let NeuralGenome::Recurrent(network) = &mut expected {
        network.memory = [0.0; 16];
    }
    let child = parent.reproduce(1, &config, &mut rng).unwrap();
    assert_eq!(child.brain, expected);
    let NeuralGenome::Recurrent(network) = &parent.brain else {
        panic!("wrong controller")
    };
    assert_eq!(network.memory, [0.5; 16]);
}

#[test]
fn recurrent_backends_agree_and_commit_memory() {
    let config = SimulationConfig {
        controller: ControllerKind::Recurrent,
        initial_population: 40,
        ..SimulationConfig::default()
    };
    let mut serial = Simulation::with_backend(config.clone(), ComputeBackend::SerialCpu);
    let mut parallel = Simulation::with_backend(config.clone(), ComputeBackend::ParallelCpu);
    let mut brute = Simulation::with_backend(config, ComputeBackend::BruteForce);
    for tick in 0..100 {
        serial.step();
        parallel.step();
        brute.step();
        for other in [&parallel, &brute] {
            assert_eq!(
                serde_json::to_string(&serial.world).unwrap(),
                serde_json::to_string(&other.world).unwrap()
            );
            for (a, b) in serial.world.organisms.iter().zip(&other.world.organisms) {
                assert_eq!(a.brain, b.brain);
                assert_eq!(a.previous_energy, b.previous_energy);
            }
        }
        if tick == 0 {
            assert!(serial.world.organisms.iter().any(|organism| matches!(&organism.brain, NeuralGenome::Recurrent(network) if network.memory.iter().any(|x| *x != 0.0))));
        }
    }
}
