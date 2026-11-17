use rayon::prelude::*;

use crate::config::SimulationConfig;
use crate::food::Food;
use crate::food_search::nearest_food_spatial;
use crate::organism::Organism;
use crate::spatial_grid::SpatialGrid;

#[derive(Debug, Clone, Copy)]
pub enum MovementAction {
    Inactive,

    SeekFood { x: f32, y: f32 },

    Wander { turn: f32 },
}

pub fn sense_food_parallel(
    organisms: &[Organism],
    food: &[Food],
    grid: &SpatialGrid,
    config: &SimulationConfig,
) -> Vec<Option<usize>> {
    organisms
        .par_iter()
        .map(|organism| {
            if !organism.is_alive(config) {
                return None;
            }

            nearest_food_spatial(organism, food, grid, config.width, config.height)
        })
        .collect()
}

pub fn execute_movement_parallel(
    organisms: &mut [Organism],
    actions: &[MovementAction],
    config: &SimulationConfig,
) {
    assert_eq!(organisms.len(), actions.len());

    organisms
        .par_iter_mut()
        .zip(actions.par_iter())
        .for_each(|(organism, action)| {
            match *action {
                MovementAction::Inactive => {
                    return;
                }

                MovementAction::SeekFood { x, y } => {
                    organism.move_towards(x, y, config);
                }

                MovementAction::Wander { turn } => {
                    organism.wander_with_turn(turn);
                }
            }

            organism.update(config);
        });
}

pub fn available_threads() -> usize {
    rayon::current_num_threads()
}
