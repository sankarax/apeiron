use apeiron_engine::{ControllerKind, NeuralGenome, SensorReading, gpu::GpuInference};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() -> Result<(), String> {
    let gpu = GpuInference::new()?;
    println!("GPU: {} ({:?})", gpu.adapter.name, gpu.adapter.backend);
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    for kind in [
        ControllerKind::Small,
        ControllerKind::Deep,
        ControllerKind::Recurrent,
    ] {
        let mut brains: Vec<_> = (0..1000)
            .map(|_| NeuralGenome::random(kind, &mut rng))
            .collect();
        for brain in &mut brains {
            if let NeuralGenome::Recurrent(n) = brain {
                n.memory = std::array::from_fn(|_| rng.random_range(-1.0..1.0));
            }
        }
        let sensors: Vec<_> = (0..brains.len())
            .map(|_| SensorReading {
                energy: rng.random(),
                age: rng.random(),
                speed: rng.random(),
                heading_x: 1.0,
                food_distance: rng.random(),
                food_direction_x: rng.random_range(-1.0..1.0),
                food_direction_y: rng.random_range(-1.0..1.0),
                food_detected: true,
                food_density: rng.random(),
                organism_density: rng.random(),
                organism_distance: rng.random(),
                organism_direction_x: rng.random_range(-1.0..1.0),
                energy_change: rng.random_range(-1.0..1.0),
                ..Default::default()
            })
            .collect();
        let refs: Vec<_> = brains.iter().collect();
        let outputs = gpu.infer(kind, &refs, &sensors)?;
        let mut max = 0.0f32;
        let mut squared = 0.0f64;
        let mut count = 0;
        for ((brain, sensor), output) in brains.iter().zip(&sensors).zip(outputs) {
            let expected = match brain {
                NeuralGenome::Small(n) => n.forward(&sensor.small_inputs()).to_vec(),
                NeuralGenome::Deep(n) => n.forward(&sensor.deep_inputs()).to_vec(),
                NeuralGenome::Recurrent(n) => {
                    let (y, h) = n.forward(&sensor.deep_inputs());
                    y.into_iter().chain(h).collect()
                }
                _ => unreachable!(),
            };
            for (a, b) in expected.iter().zip(output) {
                let error = (a - b).abs();
                assert!(error.is_finite() && error < 1e-5, "{kind:?}: {a} != {b}");
                max = max.max(error);
                squared += f64::from(error).powi(2);
                count += 1;
            }
        }
        println!(
            "{kind:?}: max error {max:.8e}, RMSE {:.8e}, {count} values passed",
            (squared / count as f64).sqrt()
        );
    }
    Ok(())
}
