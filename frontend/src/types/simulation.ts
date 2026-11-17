export interface Genome {
  speed: number;
  sensor_radius: number;
  metabolism: number;
  size: number;
  reproduction_threshold: number;
  hue: number;
}

export type Species = "herbivore" | "predator";

export interface Organism {
  species: Species;
  id: number;
  parent_id: number | null;
  generation: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  energy: number;
  age: number;
  genome: Genome;
}

export interface Food {
  id: number;
  x: number;
  y: number;
  energy: number;
}

export interface World {
  width: number;
  height: number;
  organisms: Organism[];
  food: Food[];
  next_organism_id: number;
  next_food_id: number;
}

export interface Metrics {
  tick: number;
  population: number;
  food_count: number;
  herbivores: number;
  predators: number;
  total_kills: number;
  predation_energy: number;
  total_births: number;
  total_deaths: number;
  average_energy: number;
  average_age: number;
  average_speed: number;
  average_sensor_radius: number;
  average_metabolism: number;
  average_size: number;
  average_reproduction_threshold: number;
  average_generation: number;
  max_generation: number;
}

export type ComputeBackend =
  "brute_force" | "serial_cpu" | "parallel_cpu" | "auto_cpu" | "gpu";

export interface SimulationStatus {
  running: boolean;
  tick: number;
  population: number;
  food_count: number;
  target_tps: number;
  actual_tps: number;
  backend: ComputeBackend;
  active_backend: ComputeBackend;
}

export interface SimulationSnapshot {
  type: "snapshot";
  status: SimulationStatus;
  metrics: Metrics;
  world: World;
  habitat: Pick<
    SimulationConfig,
    "predation" | "resource_zones" | "hazard_zones"
  >;
}

export type ControllerKind = "heuristic" | "small" | "deep" | "recurrent";

export interface SimulationConfig {
  predation: boolean;
  predator_fraction: number;
  predator_metabolism_scale: number;
  attack_cost: number;
  attack_range_bonus: number;
  predation_efficiency: number;
  resource_zones: boolean;
  hazard_zones: boolean;
  hazard_damage: number;
  width: number;
  height: number;
  initial_population: number;
  max_population: number;
  initial_food: number;
  max_food: number;
  food_spawn_rate: number;
  food_energy: number;
  initial_energy: number;
  max_energy: number;
  max_age: number;
  mutation_rate: number;
  mutation_strength: number;
  seed: number;
  controller: ControllerKind;
  neural_mutation_rate: number;
  neural_mutation_strength: number;
}
