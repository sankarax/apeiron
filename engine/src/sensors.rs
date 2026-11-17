use crate::config::SimulationConfig;
use crate::food::Food;
use crate::neural::DEEP_INPUTS;
use crate::organism::Organism;
use crate::spatial::wrapped_delta;
use crate::spatial_grid::SpatialGrid;

const MAX_SPEED: f32 = 6.0;
const FOOD_DENSITY_SCALE: f32 = 10.0;
const EPSILON: f32 = 1e-6;
const ORGANISM_DENSITY_SCALE: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorReading {
    pub energy: f32,
    pub age: f32,
    pub speed: f32,
    pub heading_x: f32,
    pub heading_y: f32,
    pub food_distance: f32,
    pub food_direction_x: f32,
    pub food_direction_y: f32,
    pub food_detected: bool,
    pub food_density: f32,
    pub organism_density: f32,
    pub organism_distance: f32,
    pub organism_direction_x: f32,
    pub organism_direction_y: f32,
    pub threat: f32,
    pub energy_change: f32,
}

impl Default for SensorReading {
    fn default() -> Self {
        Self {
            energy: 0.0,
            age: 0.0,
            speed: 0.0,
            heading_x: 0.0,
            heading_y: 0.0,
            food_distance: 1.0,
            food_direction_x: 0.0,
            food_direction_y: 0.0,
            food_detected: false,
            food_density: 0.0,
            organism_density: 0.0,
            organism_distance: 1.0,
            organism_direction_x: 0.0,
            organism_direction_y: 0.0,
            threat: 0.0,
            energy_change: 0.0,
        }
    }
}

impl SensorReading {
    pub fn small_inputs(&self) -> [f32; 6] {
        let (hx, hy) = if self.heading_x == 0.0 && self.heading_y == 0.0 {
            (1.0, 0.0)
        } else {
            (self.heading_x, self.heading_y)
        };

        let forward = self.food_direction_x * hx + self.food_direction_y * hy;

        let lateral = self.food_direction_y * hx - self.food_direction_x * hy;

        [
            self.energy,
            self.age,
            self.food_distance,
            forward.clamp(-1.0, 1.0),
            lateral.clamp(-1.0, 1.0),
            self.speed,
        ]
    }

    pub fn deep_inputs(&self) -> [f32; DEEP_INPUTS] {
        [
            self.energy,
            self.age,
            self.speed,
            self.heading_x,
            self.heading_y,
            self.food_distance,
            self.food_direction_x,
            self.food_direction_y,
            if self.food_detected { 1.0 } else { 0.0 },
            self.food_density,
            self.organism_density,
            self.organism_distance,
            self.organism_direction_x,
            self.organism_direction_y,
            self.threat,
            self.energy_change,
        ]
    }

    pub fn from_organism(organism: &Organism, config: &SimulationConfig) -> Self {
        let speed = organism.vx.hypot(organism.vy);
        let (heading_x, heading_y) = unit_vector(organism.vx, organism.vy);

        Self {
            energy: normalize(organism.energy, config.max_energy),
            age: normalize(organism.age as f32, config.max_age as f32),
            speed: normalize(speed, MAX_SPEED),
            heading_x,
            heading_y,
            energy_change: ((organism.energy - organism.previous_energy) / config.max_energy)
                .clamp(-1.0, 1.0),
            ..Self::default()
        }
    }

    pub fn sense_food(&mut self, organism: &Organism, food: &[Food], config: &SimulationConfig) {
        let radius = organism.genome.sensor_radius;
        let mut perception = FoodPerception::new(radius);

        for (index, item) in food.iter().enumerate() {
            perception.consider(index, item, organism, config);
        }

        perception.apply(self, radius);
    }

    pub fn sense(organism: &Organism, food: &[Food], config: &SimulationConfig) -> Self {
        let mut reading = Self::from_organism(organism, config);

        reading.sense_food(organism, food, config);

        reading
    }

    pub fn sense_food_spatial(
        &mut self,
        organism: &Organism,
        food: &[Food],
        grid: &SpatialGrid,
        config: &SimulationConfig,
    ) {
        let radius = organism.genome.sensor_radius;
        let mut perception = FoodPerception::new(radius);

        grid.for_each_candidate(organism.x, organism.y, radius, |index| {
            perception.consider(index, &food[index], organism, config);
        });

        perception.apply(self, radius);
    }

    pub fn sense_spatial(
        organism: &Organism,
        food: &[Food],
        grid: &SpatialGrid,
        config: &SimulationConfig,
    ) -> Self {
        let mut reading = Self::from_organism(organism, config);

        reading.sense_food_spatial(organism, food, grid, config);

        reading
    }

    pub fn sense_organisms(
        &mut self,
        organism: &Organism,
        organisms: &[Organism],
        config: &SimulationConfig,
    ) {
        let radius = organism.genome.sensor_radius;
        let mut perception = OrganismPerception::new(radius);

        for (index, other) in organisms.iter().enumerate() {
            perception.consider(index, other, organism, config);
        }

        perception.apply(self, radius);
    }

    pub fn sense_organisms_spatial(
        &mut self,
        organism: &Organism,
        organisms: &[Organism],
        grid: &SpatialGrid,
        config: &SimulationConfig,
    ) {
        let radius = organism.genome.sensor_radius;
        let mut perception = OrganismPerception::new(radius);

        grid.for_each_candidate(organism.x, organism.y, radius, |index| {
            perception.consider(index, &organisms[index], organism, config);
        });

        perception.apply(self, radius);
    }
}

struct FoodPerception {
    nearest_index: Option<usize>,
    nearest_squared: f32,
    nearest_direction: (f32, f32),
    visible_count: usize,
    radius_squared: f32,
}

impl FoodPerception {
    fn new(radius: f32) -> Self {
        Self {
            nearest_index: None,
            nearest_squared: radius * radius,
            nearest_direction: (0.0, 0.0),
            visible_count: 0,
            radius_squared: radius * radius,
        }
    }

    fn consider(
        &mut self,
        index: usize,
        item: &Food,
        organism: &Organism,
        config: &SimulationConfig,
    ) {
        let dx = wrapped_delta(organism.x, item.x, config.width);
        let dy = wrapped_delta(organism.y, item.y, config.height);

        let distance_squared = dx * dx + dy * dy;

        if distance_squared >= self.radius_squared {
            return;
        }

        self.visible_count += 1;

        let closer = distance_squared < self.nearest_squared;

        let earlier = distance_squared == self.nearest_squared
            && self.nearest_index.is_some_and(|nearest| index < nearest);

        if closer || earlier {
            self.nearest_index = Some(index);
            self.nearest_squared = distance_squared;
            self.nearest_direction = (dx, dy);
        }
    }

    fn apply(self, reading: &mut SensorReading, radius: f32) {
        reading.food_detected = self.nearest_index.is_some();
        reading.food_density = density(self.visible_count, FOOD_DENSITY_SCALE);

        reading.food_distance = 1.0;
        reading.food_direction_x = 0.0;
        reading.food_direction_y = 0.0;

        if reading.food_detected {
            reading.food_distance = normalize(self.nearest_squared.sqrt(), radius);

            let (dx, dy) = self.nearest_direction;
            let (direction_x, direction_y) = unit_vector(dx, dy);

            reading.food_direction_x = direction_x;
            reading.food_direction_y = direction_y;
        }
    }
}

fn normalize(value: f32, maximum: f32) -> f32 {
    if maximum <= 0.0 {
        return 0.0;
    }

    (value / maximum).clamp(0.0, 1.0)
}

fn unit_vector(x: f32, y: f32) -> (f32, f32) {
    let length = x.hypot(y);

    if length <= EPSILON {
        return (0.0, 0.0);
    }

    (x / length, y / length)
}

fn density(count: usize, scale: f32) -> f32 {
    let count = count as f32;

    count / (count + scale)
}

struct OrganismPerception {
    nearest_index: Option<usize>,
    nearest_squared: f32,
    nearest_direction: (f32, f32),
    visible_count: usize,
    radius_squared: f32,
}

impl OrganismPerception {
    fn new(radius: f32) -> Self {
        Self {
            nearest_index: None,
            nearest_squared: radius * radius,
            nearest_direction: (0.0, 0.0),
            visible_count: 0,
            radius_squared: radius * radius,
        }
    }

    fn consider(
        &mut self,
        index: usize,
        other: &Organism,
        organism: &Organism,
        config: &SimulationConfig,
    ) {
        if other.id == organism.id || !other.is_alive(config) {
            return;
        }

        let dx = wrapped_delta(organism.x, other.x, config.width);
        let dy = wrapped_delta(organism.y, other.y, config.height);

        let distance_squared = dx * dx + dy * dy;

        if distance_squared >= self.radius_squared {
            return;
        }

        self.visible_count += 1;

        let closer = distance_squared < self.nearest_squared;

        let earlier = distance_squared == self.nearest_squared
            && self.nearest_index.is_some_and(|nearest| index < nearest);

        if closer || earlier {
            self.nearest_index = Some(index);
            self.nearest_squared = distance_squared;
            self.nearest_direction = (dx, dy);
        }
    }

    fn apply(self, reading: &mut SensorReading, radius: f32) {
        reading.organism_density = density(self.visible_count, ORGANISM_DENSITY_SCALE);

        reading.organism_distance = 1.0;
        reading.organism_direction_x = 0.0;
        reading.organism_direction_y = 0.0;

        if self.nearest_index.is_some() {
            reading.organism_distance = normalize(self.nearest_squared.sqrt(), radius);

            let (dx, dy) = self.nearest_direction;
            let (x, y) = unit_vector(dx, dy);

            reading.organism_direction_x = x;
            reading.organism_direction_y = y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    #[test]
    fn default_reading_has_no_detected_food() {
        let reading = SensorReading::default();

        assert!(!reading.food_detected);
        assert_eq!(reading.food_distance, 1.0);
        assert_eq!(reading.food_direction_x, 0.0);
        assert_eq!(reading.food_direction_y, 0.0);
    }

    #[test]
    fn default_reading_has_no_nearby_organisms() {
        let reading = SensorReading::default();

        assert_eq!(reading.organism_distance, 1.0);
        assert_eq!(reading.organism_density, 0.0);
        assert_eq!(reading.organism_direction_x, 0.0);
        assert_eq!(reading.organism_direction_y, 0.0);
    }

    #[test]
    fn small_input_mapping_is_correct() {
        let reading = SensorReading {
            energy: 0.8,
            age: 0.2,
            speed: 0.5,
            heading_x: 1.0,
            heading_y: 0.0,
            food_distance: 0.3,
            food_direction_x: -0.6,
            food_direction_y: 0.8,
            ..SensorReading::default()
        };

        assert_eq!(reading.small_inputs(), [0.8, 0.2, 0.3, -0.6, 0.8, 0.5]);
    }

    #[test]
    fn deep_input_mapping_is_correct() {
        let reading = SensorReading {
            energy: 0.8,
            age: 0.2,
            speed: 0.5,
            heading_x: 0.6,
            heading_y: 0.8,
            food_distance: 0.3,
            food_direction_x: -0.6,
            food_direction_y: 0.8,
            food_detected: true,
            food_density: 0.4,
            organism_density: 0.7,
            organism_distance: 0.9,
            organism_direction_x: 0.8,
            organism_direction_y: -0.6,
            threat: 0.1,
            energy_change: -0.2,
        };

        assert_eq!(
            reading.deep_inputs(),
            [
                0.8, 0.2, 0.5, 0.6, 0.8, 0.3, -0.6, 0.8, 1.0, 0.4, 0.7, 0.9, 0.8, -0.6, 0.1, -0.2,
            ]
        );
    }

    #[test]
    fn food_detection_maps_to_binary_input() {
        let mut reading = SensorReading::default();

        assert_eq!(reading.deep_inputs()[8], 0.0);

        reading.food_detected = true;

        assert_eq!(reading.deep_inputs()[8], 1.0);
    }

    #[test]
    fn default_inputs_are_finite_and_bounded() {
        let reading = SensorReading::default();

        for input in reading.deep_inputs() {
            assert!(input.is_finite());
            assert!((-1.0..=1.0).contains(&input));
        }

        for input in reading.small_inputs() {
            assert!(input.is_finite());
            assert!((-1.0..=1.0).contains(&input));
        }
    }

    fn test_organism(config: &SimulationConfig) -> Organism {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut organism = Organism::new(0, config, &mut rng);

        organism.x = 100.0;
        organism.y = 100.0;
        organism.genome.sensor_radius = 50.0;

        organism
    }

    fn food_at(id: u64, x: f32, y: f32) -> Food {
        Food {
            id,
            x,
            y,
            energy: 25.0,
        }
    }

    #[test]
    fn internal_state_is_normalized() {
        let config = SimulationConfig::default();
        let mut organism = test_organism(&config);

        organism.energy = config.max_energy * 0.5;
        organism.age = config.max_age / 2;
        organism.vx = 3.0;
        organism.vy = 0.0;

        let reading = SensorReading::from_organism(&organism, &config);

        assert_eq!(reading.energy, 0.5);
        assert_eq!(reading.age, 0.5);
        assert_eq!(reading.speed, 0.5);
        assert_eq!(reading.heading_x, 1.0);
        assert_eq!(reading.heading_y, 0.0);
    }

    #[test]
    fn stationary_organism_has_zero_heading() {
        let config = SimulationConfig::default();
        let mut organism = test_organism(&config);

        organism.vx = 0.0;
        organism.vy = 0.0;

        let reading = SensorReading::from_organism(&organism, &config);

        assert_eq!(reading.speed, 0.0);
        assert_eq!(reading.heading_x, 0.0);
        assert_eq!(reading.heading_y, 0.0);
    }

    #[test]
    fn nearest_food_is_detected() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 140.0, 100.0), food_at(1, 125.0, 100.0)];

        let reading = SensorReading::sense(&organism, &food, &config);

        assert!(reading.food_detected);
        assert_eq!(reading.food_distance, 0.5);
        assert_eq!(reading.food_direction_x, 1.0);
        assert_eq!(reading.food_direction_y, 0.0);
    }

    #[test]
    fn distant_food_is_not_detected() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 200.0, 100.0)];

        let reading = SensorReading::sense(&organism, &food, &config);

        assert!(!reading.food_detected);
        assert_eq!(reading.food_distance, 1.0);
        assert_eq!(reading.food_density, 0.0);
    }

    #[test]
    fn food_is_detected_across_world_boundary() {
        let config = SimulationConfig::default();
        let mut organism = test_organism(&config);

        organism.x = 5.0;

        let food = [food_at(0, config.width - 5.0, organism.y)];

        let reading = SensorReading::sense(&organism, &food, &config);

        assert!(reading.food_detected);
        assert!((reading.food_distance - 0.2).abs() < 1e-6);
        assert_eq!(reading.food_direction_x, -1.0);
        assert_eq!(reading.food_direction_y, 0.0);
    }

    #[test]
    fn overlapping_food_has_zero_direction() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, organism.x, organism.y)];

        let reading = SensorReading::sense(&organism, &food, &config);

        assert!(reading.food_detected);
        assert_eq!(reading.food_distance, 0.0);
        assert_eq!(reading.food_direction_x, 0.0);
        assert_eq!(reading.food_direction_y, 0.0);
    }

    #[test]
    fn food_density_counts_visible_food_only() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let mut food: Vec<_> = (0..10).map(|id| food_at(id, 110.0, 100.0)).collect();

        food.push(food_at(10, 300.0, 100.0));

        let reading = SensorReading::sense(&organism, &food, &config);

        assert_eq!(reading.food_density, 0.5);
    }

    #[test]
    fn food_at_exact_radius_is_excluded() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 150.0, 100.0)];

        let reading = SensorReading::sense(&organism, &food, &config);

        assert!(!reading.food_detected);
        assert_eq!(reading.food_density, 0.0);
    }

    #[test]
    fn equal_distance_food_preserves_first_item() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 125.0, 100.0), food_at(1, 75.0, 100.0)];

        let reading = SensorReading::sense(&organism, &food, &config);

        assert_eq!(reading.food_direction_x, 1.0);
    }

    #[test]
    fn repeated_sensing_clears_previous_food() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 125.0, 100.0)];

        let mut reading = SensorReading::sense(&organism, &food, &config);

        assert!(reading.food_detected);

        reading.sense_food(&organism, &[], &config);

        assert!(!reading.food_detected);
        assert_eq!(reading.food_distance, 1.0);
        assert_eq!(reading.food_direction_x, 0.0);
        assert_eq!(reading.food_density, 0.0);
    }

    #[test]
    fn calculated_inputs_are_finite_and_bounded() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 120.0, 130.0), food_at(1, 90.0, 90.0)];

        let reading = SensorReading::sense(&organism, &food, &config);

        for input in reading.deep_inputs() {
            assert!(input.is_finite());
            assert!((-1.0..=1.0).contains(&input));
        }
    }

    fn food_grid(food: &[Food], config: &SimulationConfig) -> SpatialGrid {
        let mut grid = SpatialGrid::new(config.width, config.height, 50.0);

        grid.rebuild_food(food);

        grid
    }

    fn assert_food_sensors_match(organism: &Organism, food: &[Food], config: &SimulationConfig) {
        let grid = food_grid(food, config);

        let brute = SensorReading::sense(organism, food, config);

        let spatial = SensorReading::sense_spatial(organism, food, &grid, config);

        assert_eq!(brute, spatial);
    }

    #[test]
    fn spatial_food_sensing_matches_brute_force() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [
            food_at(0, 120.0, 100.0),
            food_at(1, 110.0, 110.0),
            food_at(2, 300.0, 300.0),
        ];

        assert_food_sensors_match(&organism, &food, &config);
    }

    #[test]
    fn spatial_food_sensing_handles_empty_world() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        assert_food_sensors_match(&organism, &[], &config);
    }

    #[test]
    fn spatial_food_sensing_wraps_boundaries() {
        let config = SimulationConfig::default();
        let mut organism = test_organism(&config);

        organism.x = 5.0;
        organism.y = 5.0;

        let food = [
            food_at(0, config.width - 5.0, 5.0),
            food_at(1, 5.0, config.height - 5.0),
            food_at(2, config.width - 2.0, config.height - 2.0),
        ];

        assert_food_sensors_match(&organism, &food, &config);
    }

    #[test]
    fn spatial_food_sensing_preserves_ties() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 125.0, 100.0), food_at(1, 75.0, 100.0)];

        assert_food_sensors_match(&organism, &food, &config);
    }

    #[test]
    fn spatial_food_sensing_excludes_exact_radius() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [food_at(0, 150.0, 100.0), food_at(1, 50.0, 100.0)];

        assert_food_sensors_match(&organism, &food, &config);
    }

    #[test]
    fn spatial_food_sensing_handles_overlapping_food() {
        let config = SimulationConfig::default();
        let organism = test_organism(&config);

        let food = [
            food_at(0, organism.x, organism.y),
            food_at(1, organism.x, organism.y),
        ];

        assert_food_sensors_match(&organism, &food, &config);
    }

    #[test]
    fn spatial_food_sensing_handles_large_radius() {
        let config = SimulationConfig {
            width: 103.0,
            height: 77.0,
            ..SimulationConfig::default()
        };

        let mut organism = test_organism(&config);

        organism.x = 20.0;
        organism.y = 20.0;
        organism.genome.sensor_radius = 200.0;

        let food = [
            food_at(0, 10.0, 10.0),
            food_at(1, 50.0, 50.0),
            food_at(2, 100.0, 70.0),
        ];

        assert_food_sensors_match(&organism, &food, &config);
    }

    #[test]
    fn spatial_food_sensing_matches_random_worlds() {
        use rand::Rng;

        for seed in 0..10 {
            let config = SimulationConfig::default();
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            let food: Vec<_> = (0..1000)
                .map(|id| {
                    food_at(
                        id,
                        rng.random_range(0.0..config.width),
                        rng.random_range(0.0..config.height),
                    )
                })
                .collect();

            let grid = food_grid(&food, &config);

            for id in 0..100 {
                let organism = Organism::new(id, &config, &mut rng);

                let brute = SensorReading::sense(&organism, &food, &config);

                let spatial = SensorReading::sense_spatial(&organism, &food, &grid, &config);

                assert_eq!(brute, spatial);
            }
        }
    }
}
