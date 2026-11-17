use apeiron_engine::{ControllerKind, Simulation, SimulationConfig};
fn main() {
    for seed in [42, 123, 456] {
        let config = SimulationConfig {
            predation: true,
            controller: ControllerKind::Recurrent,
            initial_population: 1000,
            seed,
            ..Default::default()
        };
        let mut sim = Simulation::new(config);
        for _ in 0..10 {
            sim.run(500);
            let m = sim.metrics();
            assert_eq!(m.population, m.predators + m.herbivores);
            assert_eq!(m.population as u64, 1000 + m.total_births - m.total_deaths);
            assert!(m.total_kills <= m.total_deaths);
            println!(
                "seed {seed} tick {} herbivores {} predators {} kills {} energy {:.2}",
                m.tick, m.herbivores, m.predators, m.total_kills, m.predation_energy
            );
        }
    }
}
