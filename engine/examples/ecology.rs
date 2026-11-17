use apeiron_engine::{ControllerKind, Simulation, SimulationConfig};

fn main() {
    for (name, cost, scale, reach, zones, hazards) in [
        ("baseline", 1.0, 1.0, 2.0, false, false),
        ("cheap_attack", 0.1, 1.0, 2.0, false, false),
        ("low_metabolism", 1.0, 0.4, 2.0, false, false),
        ("long_reach", 1.0, 1.0, 8.0, false, false),
        ("combined", 0.1, 0.4, 8.0, false, false),
        ("complex", 0.1, 0.4, 8.0, true, true),
    ] {
        for seed in [42, 123, 456] {
            let config = SimulationConfig {
                predation: true,
                controller: ControllerKind::Recurrent,
                initial_population: 1000,
                seed,
                attack_cost: cost,
                predator_metabolism_scale: scale,
                attack_range_bonus: reach,
                resource_zones: zones,
                hazard_zones: hazards,
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
                    "{name} seed {seed} tick {} herbivores {} predators {} kills {} energy {:.2}",
                    m.tick, m.herbivores, m.predators, m.total_kills, m.predation_energy
                );
            }
        }
    }
}
