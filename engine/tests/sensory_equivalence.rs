use apeiron_engine::{
    ControllerKind, Food, Organism, SensorReading, SensoryContext, SimulationConfig, SpatialGrid,
    World,
};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn config() -> SimulationConfig {
    SimulationConfig {
        controller: ControllerKind::Deep,
        initial_population: 200,
        initial_food: 500,
        ..SimulationConfig::default()
    }
}

fn organism_at(id: u64, x: f32, y: f32, config: &SimulationConfig) -> Organism {
    let mut rng = ChaCha8Rng::seed_from_u64(id);

    let mut organism = Organism::new(id, config, &mut rng);

    organism.x = x;
    organism.y = y;
    organism.genome.sensor_radius = 50.0;

    organism
}

#[test]
fn organism_sensing_excludes_self() {
    let config = config();
    let organisms = [organism_at(0, 100.0, 100.0, &config)];

    let mut reading = SensorReading::default();

    reading.sense_organisms(&organisms[0], &organisms, &config);

    assert_eq!(reading.organism_density, 0.0);
    assert_eq!(reading.organism_distance, 1.0);
}

#[test]
fn nearest_organism_is_detected() {
    let config = config();

    let organisms = [
        organism_at(0, 100.0, 100.0, &config),
        organism_at(1, 125.0, 100.0, &config),
        organism_at(2, 140.0, 100.0, &config),
    ];

    let mut reading = SensorReading::default();

    reading.sense_organisms(&organisms[0], &organisms, &config);

    assert_eq!(reading.organism_distance, 0.5);
    assert_eq!(reading.organism_direction_x, 1.0);
    assert_eq!(reading.organism_direction_y, 0.0);
    assert!((reading.organism_density - 2.0 / 7.0).abs() < 1e-6);
}

#[test]
fn dead_organisms_are_excluded() {
    let config = config();

    let mut organisms = [
        organism_at(0, 100.0, 100.0, &config),
        organism_at(1, 110.0, 100.0, &config),
    ];

    organisms[1].energy = 0.0;

    let mut reading = SensorReading::default();

    reading.sense_organisms(&organisms[0], &organisms, &config);

    assert_eq!(reading.organism_density, 0.0);
    assert_eq!(reading.organism_distance, 1.0);
}

#[test]
fn organism_perception_wraps_boundaries() {
    let config = config();

    let organisms = [
        organism_at(0, 5.0, 100.0, &config),
        organism_at(1, config.width - 5.0, 100.0, &config),
    ];

    let mut reading = SensorReading::default();

    reading.sense_organisms(&organisms[0], &organisms, &config);

    assert!((reading.organism_distance - 0.2).abs() < 1e-6);
    assert_eq!(reading.organism_direction_x, -1.0);
}

#[test]
fn energy_history_tracks_changes() {
    let config = config();
    let mut organism = organism_at(0, 100.0, 100.0, &config);

    organism.energy = 100.0;
    organism.record_energy();
    organism.energy = 125.0;

    let reading = SensorReading::from_organism(&organism, &config);

    assert_eq!(reading.energy_change, 25.0 / config.max_energy);

    organism.energy = 75.0;

    let reading = SensorReading::from_organism(&organism, &config);

    assert_eq!(reading.energy_change, -25.0 / config.max_energy);
}

#[test]
fn sensory_backends_match() {
    for seed in 0..5 {
        for controller in [ControllerKind::Small, ControllerKind::Deep] {
            let config = SimulationConfig {
                seed,
                controller,
                ..config()
            };

            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let world = World::new(&config, &mut rng);

            let mut food_grid = SpatialGrid::new(config.width, config.height, 50.0);

            let mut organism_grid = SpatialGrid::new(config.width, config.height, 50.0);

            food_grid.rebuild_food(&world.food);
            organism_grid.rebuild_organisms(&world.organisms);

            let context = SensoryContext {
                organisms: &world.organisms,
                food: &world.food,
                organism_grid: &organism_grid,
                food_grid: &food_grid,
                config: &config,
            };

            let brute = context.collect_brute_force();
            let serial = context.collect_serial();
            let parallel = context.collect_parallel();

            assert_eq!(brute, serial);
            assert_eq!(serial, parallel);

            for reading in parallel {
                for input in reading.deep_inputs() {
                    assert!(input.is_finite());
                    assert!((-1.0..=1.0).contains(&input));
                }
            }
        }
    }
}

#[test]
fn sensory_pipeline_handles_empty_world() {
    let config = config();

    let organisms: Vec<Organism> = Vec::new();
    let food: Vec<Food> = Vec::new();

    let grid = SpatialGrid::new(config.width, config.height, 50.0);

    let context = SensoryContext {
        organisms: &organisms,
        food: &food,
        organism_grid: &grid,
        food_grid: &grid,
        config: &config,
    };

    assert!(context.collect_serial().is_empty());
    assert!(context.collect_parallel().is_empty());
}
