use crate::organism::Species;
use crate::{Genome, Organism};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRecord {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub generation: u32,
    pub species: Species,
    pub birth_tick: u64,
    pub death_tick: Option<u64>,
    pub genome: Genome,
}

impl LineageRecord {
    pub(crate) fn new(organism: &Organism, tick: u64) -> Self {
        Self {
            id: organism.id,
            parent_id: organism.parent_id,
            generation: organism.generation,
            species: organism.species,
            birth_tick: tick,
            death_tick: None,
            genome: organism.genome.clone(),
        }
    }
}
