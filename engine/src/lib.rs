pub mod config;
pub mod food;
pub mod genome;
pub mod lineage;
pub mod metrics;
pub use lineage::LineageRecord;
pub mod organism;
pub mod simulation;
pub mod spatial;
pub mod spatial_grid;
pub mod world;
pub use spatial_grid::SpatialGrid;
pub mod compute_backend;
pub mod food_search;
pub use compute_backend::ComputeBackend;
pub mod parallel;
pub mod profiling;

pub mod neural;

pub mod neural_controller;
pub use neural_controller::NeuralAction;
pub mod neural_genome;
pub mod neural_mutation;
pub mod sensors;
pub mod sensory_pipeline;

pub use config::SimulationConfig;
pub use food::Food;
pub use genome::Genome;
pub use metrics::Metrics;
pub use neural::{DeepNetwork, DenseLayer, SmallNetwork};
pub use neural_genome::{ControllerKind, NeuralGenome};
pub use neural_mutation::NeuralMutation;
pub use organism::Organism;
pub use profiling::TickProfile;
pub use sensors::SensorReading;
pub use sensory_pipeline::SensoryContext;
pub use simulation::Simulation;
pub use world::World;

#[cfg(feature = "python")]
mod python;

pub mod recurrent;
pub use recurrent::{HiddenState, RecurrentNetwork};

#[cfg(not(target_arch = "wasm32"))]
pub mod gpu;

#[cfg(target_arch = "wasm32")]
mod browser;
