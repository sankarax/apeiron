use apeiron_engine::{ComputeBackend, ControllerKind, Simulation, SimulationConfig};

#[test]
fn deterministic_checkpoint_replay() {
    for controller in [
        ControllerKind::Heuristic,
        ControllerKind::Small,
        ControllerKind::Deep,
        ControllerKind::Recurrent,
    ] {
        let config = SimulationConfig {
            controller,
            ..SimulationConfig::default()
        };
        let mut sim = Simulation::with_backend(config, ComputeBackend::ParallelCpu);
        sim.run(1000);
        let saved = sim.checkpoint().unwrap();
        let mut restored = Simulation::from_checkpoint(&saved).unwrap();
        assert_eq!(saved, restored.checkpoint().unwrap());
        sim.run(1000);
        restored.run(1000);
        assert_eq!(sim.checkpoint().unwrap(), restored.checkpoint().unwrap());
        assert_eq!(sim.lineage().len() as u64, 200 + sim.total_births);
        assert_eq!(
            sim.lineage()
                .values()
                .filter(|r| r.death_tick.is_some())
                .count() as u64,
            sim.total_deaths
        );
    }
}

#[test]
fn predation_checkpoint_replay() {
    let config = SimulationConfig {
        controller: ControllerKind::Recurrent,
        predation: true,
        predator_metabolism_scale: 0.4,
        resource_zones: true,
        hazard_zones: true,
        ..SimulationConfig::default()
    };
    let mut sim = Simulation::new(config);
    sim.run(300);
    let mut restored = Simulation::from_checkpoint(&sim.checkpoint().unwrap()).unwrap();
    sim.run(300);
    restored.run(300);
    assert_eq!(sim.checkpoint().unwrap(), restored.checkpoint().unwrap());
}

#[test]
fn rejects_invalid_checkpoint() {
    assert!(Simulation::from_checkpoint("{}").is_err());
    let sim = Simulation::new(SimulationConfig::default());
    let mut state: serde_json::Value = serde_json::from_str(&sim.checkpoint().unwrap()).unwrap();
    state["version"] = 99.into();
    assert!(Simulation::from_checkpoint(&state.to_string()).is_err());
}
