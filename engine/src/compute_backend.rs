use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputeBackend {
    BruteForce,
    SerialCpu,
    ParallelCpu,
    #[default]
    AutoCpu,
}

const PARALLEL_THRESHOLD: usize = 500;

impl ComputeBackend {
    pub fn resolve(self, population: usize) -> Self {
        #[cfg(target_arch = "wasm32")]
        if matches!(self, Self::AutoCpu | Self::ParallelCpu) {
            return Self::SerialCpu;
        }
        match self {
            Self::AutoCpu if population >= PARALLEL_THRESHOLD => Self::ParallelCpu,
            Self::AutoCpu => Self::SerialCpu,
            backend => backend,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BruteForce => "brute_force",
            Self::SerialCpu => "serial_cpu",
            Self::ParallelCpu => "parallel_cpu",
            Self::AutoCpu => "auto_cpu",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "brute_force" => Some(Self::BruteForce),
            "serial_cpu" => Some(Self::SerialCpu),
            "parallel_cpu" => Some(Self::ParallelCpu),
            "auto_cpu" => Some(Self::AutoCpu),
            _ => None,
        }
    }
}
