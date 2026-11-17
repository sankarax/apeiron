use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct TickProfile {
    pub spawning: Duration,
    pub grid: Duration,
    pub movement: Duration,
    pub feeding: Duration,
    pub reproduction: Duration,
    pub cleanup: Duration,
}

impl TickProfile {
    pub fn total(&self) -> Duration {
        self.spawning + self.grid + self.movement + self.feeding + self.reproduction + self.cleanup
    }

    pub fn accumulate(&mut self, other: &Self) {
        self.spawning += other.spawning;
        self.grid += other.grid;
        self.movement += other.movement;
        self.feeding += other.feeding;
        self.reproduction += other.reproduction;
        self.cleanup += other.cleanup;
    }
}
