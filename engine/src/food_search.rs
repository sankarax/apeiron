use crate::food::Food;
use crate::organism::Organism;
use crate::spatial::distance_squared;
use crate::spatial_grid::SpatialGrid;

// brute force reference
pub fn nearest_food_brute_force(
    organism: &Organism,
    food: &[Food],
    width: f32,
    height: f32,
) -> Option<usize> {
    let mut nearest_index = None;

    let mut nearest_distance = organism.genome.sensor_radius.powi(2);

    for (index, resource) in food.iter().enumerate() {
        let distance = distance_squared(
            organism.x, organism.y, resource.x, resource.y, width, height,
        );

        if distance < nearest_distance {
            nearest_distance = distance;
            nearest_index = Some(index);
        }
    }

    nearest_index
}

// spatial grid search
pub fn nearest_food_spatial(
    organism: &Organism,
    food: &[Food],
    grid: &SpatialGrid,
    width: f32,
    height: f32,
) -> Option<usize> {
    let radius = organism.genome.sensor_radius;

    let mut nearest_index: Option<usize> = None;

    let mut nearest_distance = radius.powi(2);

    grid.for_each_candidate(organism.x, organism.y, radius, |index| {
        let resource = &food[index];

        let distance = distance_squared(
            organism.x, organism.y, resource.x, resource.y, width, height,
        );

        let closer = distance < nearest_distance;

        let equal_but_earlier =
            distance == nearest_distance && nearest_index.is_some_and(|previous| index < previous);

        if closer || equal_but_earlier {
            nearest_distance = distance;
            nearest_index = Some(index);
        }
    });

    nearest_index
}

pub fn nearest_edible_food_brute_force(
    organism: &Organism,
    food: &[Food],
    consumed: &[bool],
    width: f32,
    height: f32,
) -> Option<usize> {
    let radius = organism.genome.size + 2.0;

    let mut nearest_distance = radius.powi(2);

    let mut nearest_index = None;

    for (index, resource) in food.iter().enumerate() {
        if consumed[index] {
            continue;
        }

        let distance = distance_squared(
            organism.x, organism.y, resource.x, resource.y, width, height,
        );

        if distance <= nearest_distance {
            nearest_distance = distance;
            nearest_index = Some(index);
        }
    }

    nearest_index
}

pub fn nearest_edible_food_spatial(
    organism: &Organism,
    food: &[Food],
    consumed: &[bool],
    grid: &SpatialGrid,
    width: f32,
    height: f32,
) -> Option<usize> {
    let radius = organism.genome.size + 2.0;

    let mut nearest_distance = radius.powi(2);

    let mut nearest_index: Option<usize> = None;

    grid.for_each_candidate(organism.x, organism.y, radius, |index| {
        if consumed[index] {
            return;
        }

        let resource = &food[index];

        let distance = distance_squared(
            organism.x, organism.y, resource.x, resource.y, width, height,
        );

        let closer = distance < nearest_distance;

        let equal_but_later =
            distance == nearest_distance && nearest_index.is_none_or(|previous| index > previous);

        if closer || equal_but_later {
            nearest_distance = distance;
            nearest_index = Some(index);
        }
    });

    nearest_index
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Simulation, SimulationConfig};

    #[test]
    fn spatial_matches_brute_force() {
        for seed in 0..10 {
            let config = SimulationConfig {
                initial_population: 500,

                initial_food: 1000,

                seed,

                ..SimulationConfig::default()
            };

            let simulation = Simulation::new(config);

            let world = &simulation.world;

            let mut grid = SpatialGrid::new(world.width, world.height, 50.0);

            grid.rebuild_food(&world.food);

            for organism in &world.organisms {
                let brute_force =
                    nearest_food_brute_force(organism, &world.food, world.width, world.height);

                let spatial =
                    nearest_food_spatial(organism, &world.food, &grid, world.width, world.height);

                assert_eq!(
                    brute_force, spatial,
                    "Mismatch: seed={}, organism={}",
                    seed, organism.id,
                );
            }
        }
    }

    #[test]
    fn spatial_search_handles_empty_food() {
        let simulation = Simulation::new(SimulationConfig::default());

        let organism = &simulation.world.organisms[0];

        let grid = SpatialGrid::new(1600.0, 900.0, 50.0);

        let result = nearest_food_spatial(organism, &[], &grid, 1600.0, 900.0);

        assert_eq!(result, None);
    }

    #[test]
    fn spatial_search_preserves_tie_breaking() {
        let simulation = Simulation::new(SimulationConfig::default());

        let mut organism = simulation.world.organisms[0].clone();

        organism.x = 100.0;
        organism.y = 100.0;

        organism.genome.sensor_radius = 100.0;

        let food = vec![
            Food {
                id: 0,
                x: 150.0,
                y: 100.0,
                energy: 25.0,
            },
            Food {
                id: 1,
                x: 50.0,
                y: 100.0,
                energy: 25.0,
            },
        ];

        let mut grid = SpatialGrid::new(1600.0, 900.0, 50.0);

        grid.rebuild_food(&food);

        let result = nearest_food_spatial(&organism, &food, &grid, 1600.0, 900.0);

        assert_eq!(result, Some(0));
    }

    #[test]
    fn spatial_search_finds_food_across_boundary() {
        let simulation = Simulation::new(SimulationConfig::default());

        let mut organism = simulation.world.organisms[0].clone();

        organism.x = 2.0;
        organism.y = 450.0;

        organism.genome.sensor_radius = 20.0;

        let food = vec![
            Food {
                id: 0,
                x: 1598.0,
                y: 450.0,
                energy: 25.0,
            },
            Food {
                id: 1,
                x: 15.0,
                y: 450.0,
                energy: 25.0,
            },
        ];

        let mut grid = SpatialGrid::new(1600.0, 900.0, 50.0);

        grid.rebuild_food(&food);

        let result = nearest_food_spatial(&organism, &food, &grid, 1600.0, 900.0);

        assert_eq!(result, Some(0));
    }

    #[test]
    fn edible_spatial_matches_brute_force() {
        for seed in 0..10 {
            let config = SimulationConfig {
                initial_population: 500,
                initial_food: 1000,
                seed,
                ..SimulationConfig::default()
            };

            let simulation = Simulation::new(config);

            let world = &simulation.world;

            let mut grid = SpatialGrid::new(world.width, world.height, 50.0);

            grid.rebuild_food(&world.food);

            let consumed: Vec<bool> = (0..world.food.len()).map(|index| index % 3 == 0).collect();

            for organism in &world.organisms {
                let brute_force = nearest_edible_food_brute_force(
                    organism,
                    &world.food,
                    &consumed,
                    world.width,
                    world.height,
                );

                let spatial = nearest_edible_food_spatial(
                    organism,
                    &world.food,
                    &consumed,
                    &grid,
                    world.width,
                    world.height,
                );

                assert_eq!(
                    brute_force, spatial,
                    "Feeding mismatch: seed={}, organism={}",
                    seed, organism.id,
                );
            }
        }
    }

    #[test]
    fn edible_search_skips_consumed_food() {
        let simulation = Simulation::new(SimulationConfig::default());

        let mut organism = simulation.world.organisms[0].clone();

        organism.x = 100.0;
        organism.y = 100.0;
        organism.genome.size = 5.0;

        let food = vec![
            Food {
                id: 0,
                x: 101.0,
                y: 100.0,
                energy: 25.0,
            },
            Food {
                id: 1,
                x: 104.0,
                y: 100.0,
                energy: 25.0,
            },
        ];

        let consumed = vec![true, false];

        let mut grid = SpatialGrid::new(1600.0, 900.0, 50.0);

        grid.rebuild_food(&food);

        let result = nearest_edible_food_spatial(&organism, &food, &consumed, &grid, 1600.0, 900.0);

        assert_eq!(result, Some(1));
    }

    #[test]
    fn edible_search_preserves_later_index_tie() {
        let simulation = Simulation::new(SimulationConfig::default());

        let mut organism = simulation.world.organisms[0].clone();

        organism.x = 100.0;
        organism.y = 100.0;
        organism.genome.size = 5.0;

        let food = vec![
            Food {
                id: 0,
                x: 104.0,
                y: 100.0,
                energy: 25.0,
            },
            Food {
                id: 1,
                x: 96.0,
                y: 100.0,
                energy: 25.0,
            },
        ];

        let consumed = vec![false, false];

        let mut grid = SpatialGrid::new(1600.0, 900.0, 50.0);

        grid.rebuild_food(&food);

        let result = nearest_edible_food_spatial(&organism, &food, &consumed, &grid, 1600.0, 900.0);

        assert_eq!(result, Some(1));
    }

    #[test]
    fn edible_search_includes_exact_radius() {
        let simulation = Simulation::new(SimulationConfig::default());

        let mut organism = simulation.world.organisms[0].clone();

        organism.x = 100.0;
        organism.y = 100.0;
        organism.genome.size = 5.0;

        // eating radius is 7

        let food = vec![Food {
            id: 0,
            x: 107.0,
            y: 100.0,
            energy: 25.0,
        }];

        let consumed = vec![false];

        let mut grid = SpatialGrid::new(1600.0, 900.0, 50.0);

        grid.rebuild_food(&food);

        let result = nearest_edible_food_spatial(&organism, &food, &consumed, &grid, 1600.0, 900.0);

        assert_eq!(result, Some(0));
    }
}
