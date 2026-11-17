use apeiron_engine::{
    ControllerKind, SensorReading, Simulation, SimulationConfig, gpu::GpuInference,
};
fn main() -> Result<(), String> {
    let gpu = GpuInference::new()?;
    println!("GPU: {} ({:?})", gpu.adapter.name, gpu.adapter.backend);
    for kind in [
        ControllerKind::Small,
        ControllerKind::Deep,
        ControllerKind::Recurrent,
    ] {
        for empty in [false, true] {
            let config = SimulationConfig {
                controller: kind,
                initial_population: 1000,
                initial_food: if empty { 0 } else { 1000 },
                ..Default::default()
            };
            let mut sim = Simulation::new(config);
            sim.world.organisms[0].x = 1.0;
            sim.world.organisms[1].x = sim.config.width - 1.0;
            sim.world.organisms[1].y = sim.world.organisms[0].y;
            sim.world.organisms[2].energy = 0.0;
            let actual = gpu.sense(&sim.world.organisms, &sim.world.food, &sim.config)?;
            let mut max = 0.0f32;
            let mut failures = 0;
            for (o, a) in sim.world.organisms.iter().zip(actual) {
                let mut expected = SensorReading::sense(o, &sim.world.food, &sim.config);
                if kind != ControllerKind::Small {
                    expected.sense_organisms(o, &sim.world.organisms, &sim.config);
                }
                for (field, (x, y)) in expected
                    .deep_inputs()
                    .into_iter()
                    .zip(a.deep_inputs())
                    .enumerate()
                {
                    let error = (x - y).abs();
                    if !error.is_finite() || error >= 1e-4 {
                        failures += 1;
                        if failures <= 5 {
                            println!("{kind:?} organism {} field {field}: {x} != {y}", o.id);
                        }
                    }
                    max = max.max(error);
                }
            }
            if failures != 0 {
                let full =
                    gpu.sense_mode(&sim.world.organisms, &sim.world.food, &sim.config, true)?;
                let mut full_failures = 0;
                for (o, a) in sim.world.organisms.iter().zip(full) {
                    let mut expected = SensorReading::sense(o, &sim.world.food, &sim.config);
                    if kind != ControllerKind::Small {
                        expected.sense_organisms(o, &sim.world.organisms, &sim.config);
                    }
                    for (x, y) in expected.deep_inputs().into_iter().zip(a.deep_inputs()) {
                        if !(x - y).is_finite() || (x - y).abs() >= 1e-4 {
                            full_failures += 1;
                        }
                    }
                }
                return Err(format!(
                    "spatial mismatches: {failures}; all-cell mismatches: {full_failures}"
                ));
            }
            println!("{kind:?}, empty food={empty}: max error {max:.8e}, 16000 values passed");
        }
    }
    Ok(())
}
