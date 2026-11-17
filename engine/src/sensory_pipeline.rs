use rayon::prelude::*;

use crate::config::SimulationConfig;
use crate::food::Food;
use crate::neural_genome::ControllerKind;
use crate::organism::Organism;
use crate::sensors::SensorReading;
use crate::spatial_grid::SpatialGrid;

pub struct SensoryContext<'a> {
    pub organisms: &'a [Organism],
    pub food: &'a [Food],
    pub organism_grid: &'a SpatialGrid,
    pub food_grid: &'a SpatialGrid,
    pub config: &'a SimulationConfig,
}

impl SensoryContext<'_> {
    pub fn read(&self, organism: &Organism) -> SensorReading {
        let mut reading =
            SensorReading::sense_spatial(organism, self.food, self.food_grid, self.config);

        if matches!(
            organism.brain.kind(),
            ControllerKind::Deep | ControllerKind::Recurrent
        ) {
            reading.sense_organisms_spatial(
                organism,
                self.organisms,
                self.organism_grid,
                self.config,
            );
        }

        if self.config.predation {
            apply_species_sensors(
                &mut reading,
                organism,
                self.organisms,
                Some(self.organism_grid),
                self.config,
            );
        }
        reading
    }

    pub fn read_brute_force(&self, organism: &Organism) -> SensorReading {
        let mut reading = SensorReading::sense(organism, self.food, self.config);

        if matches!(
            organism.brain.kind(),
            ControllerKind::Deep | ControllerKind::Recurrent
        ) {
            reading.sense_organisms(organism, self.organisms, self.config);
        }

        if self.config.predation {
            apply_species_sensors(&mut reading, organism, self.organisms, None, self.config);
        }
        reading
    }

    pub fn collect_serial(&self) -> Vec<SensorReading> {
        self.organisms
            .iter()
            .map(|organism| self.read(organism))
            .collect()
    }

    pub fn collect_parallel(&self) -> Vec<SensorReading> {
        self.organisms
            .par_iter()
            .map(|organism| self.read(organism))
            .collect()
    }

    pub fn collect_brute_force(&self) -> Vec<SensorReading> {
        self.organisms
            .iter()
            .map(|organism| self.read_brute_force(organism))
            .collect()
    }
}

// use the same species perception for cpu and gpu inference
pub(crate) fn apply_species_sensors(
    reading: &mut SensorReading,
    organism: &Organism,
    organisms: &[Organism],
    grid: Option<&SpatialGrid>,
    config: &SimulationConfig,
) {
    use crate::organism::Species;
    use crate::spatial::wrapped_delta;
    let radius = organism.genome.sensor_radius;
    let mut nearest = None;
    let mut best = radius * radius;
    let mut count = 0u32;
    let mut visit = |i: usize| {
        let other = &organisms[i];
        if other.species == organism.species || !other.is_alive(config) {
            return;
        }
        let dx = wrapped_delta(organism.x, other.x, config.width);
        let dy = wrapped_delta(organism.y, other.y, config.height);
        let d = dx * dx + dy * dy;
        if d >= radius * radius {
            return;
        }
        count += 1;
        if d < best || (d == best && nearest.is_some_and(|(j, _, _)| i < j)) {
            best = d;
            nearest = Some((i, dx, dy));
        }
    };
    if let Some(grid) = grid {
        grid.for_each_candidate(organism.x, organism.y, radius, visit);
    } else {
        for i in 0..organisms.len() {
            visit(i);
        }
    }
    let distance = best.sqrt();
    reading.organism_distance = (distance / radius).clamp(0.0, 1.0);
    reading.organism_density = count as f32 / (count as f32 + 5.0);
    let (dx, dy) = nearest.map_or((0.0, 0.0), |(_, x, y)| (x, y));
    reading.organism_direction_x = if distance > 1e-6 { dx / distance } else { 0.0 };
    reading.organism_direction_y = if distance > 1e-6 { dy / distance } else { 0.0 };
    reading.threat = if organism.species == Species::Herbivore {
        1.0 - reading.organism_distance
    } else {
        0.0
    };
    if organism.species == Species::Predator {
        reading.food_distance = 1.0;
        reading.food_direction_x = 0.0;
        reading.food_direction_y = 0.0;
        reading.food_density = 0.0;
        reading.food_detected = false;
    }
}
