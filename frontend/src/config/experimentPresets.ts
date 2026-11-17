import type { SimulationConfig } from "../types/simulation";

export const DEFAULT_CONFIG: SimulationConfig = {
  predation: false,
  predator_fraction: 0.2,
  predator_metabolism_scale: 1,
  attack_cost: 1,
  attack_range_bonus: 2,
  predation_efficiency: 0.7,
  resource_zones: false,
  hazard_zones: false,
  hazard_damage: 0.5,
  width: 1600,
  height: 900,

  initial_population: 200,
  max_population: 5000,

  initial_food: 500,
  max_food: 2000,
  food_spawn_rate: 10,
  food_energy: 25,

  initial_energy: 100,
  max_energy: 250,
  max_age: 3000,

  mutation_rate: 0.1,
  mutation_strength: 0.15,
  controller: "heuristic",
  neural_mutation_rate: 0.05,
  neural_mutation_strength: 0.1,
  seed: 42,
};

export interface ExperimentPreset {
  id: string;
  name: string;
  description: string;
  config: Partial<SimulationConfig>;
}

export const EXPERIMENT_PRESETS: ExperimentPreset[] = [
  {
    id: "balanced",

    name: "Balanced ecosystem",

    description: "Standard environmental conditions for baseline experiments.",

    config: {},
  },

  {
    id: "scarcity",

    name: "Resource scarcity",

    description: "Limited resources increase competition for survival.",

    config: {
      initial_food: 200,
      max_food: 500,
      food_spawn_rate: 3,
      food_energy: 20,
    },
  },

  {
    id: "abundance",

    name: "Abundant resources",

    description: "High resource availability reduces food competition.",

    config: {
      initial_food: 1500,
      max_food: 3000,
      food_spawn_rate: 25,
      food_energy: 35,
    },
  },

  {
    id: "hypermutation",

    name: "Hypermutation",

    description: "Increased mutation probability and genetic variation.",

    config: {
      mutation_rate: 0.5,
      mutation_strength: 0.4,
    },
  },

  {
    id: "stability",

    name: "Genetic stability",

    description:
      "Natural selection acts on existing genetic variation without new mutations.",

    config: {
      mutation_rate: 0,
      mutation_strength: 0,
    },
  },

  {
    id: "competition",

    name: "Population pressure",

    description: "A dense initial population competes for limited resources.",

    config: {
      initial_population: 1000,
      max_population: 3000,
      initial_food: 500,
      food_spawn_rate: 8,
    },
  },
  ...[
    {
      id: "predation-baseline",
      name: "Predation baseline",
      description: "Original predator extinction experiment.",
      config: {},
    },
    {
      id: "cheap-attack",
      name: "Cheap attacks",
      description: "Reduced attack energy cost.",
      config: { attack_cost: 0.1 },
    },
    {
      id: "coexistence",
      name: "Predator-prey coexistence",
      description:
        "Reduced predator metabolism; both species survived 5,000 ticks in three tested seeds.",
      config: { predator_metabolism_scale: 0.4 },
    },
    {
      id: "long-reach",
      name: "Long reach",
      description: "Increased attack range.",
      config: { attack_range_bonus: 8 },
    },
    {
      id: "combined",
      name: "Intense predation",
      description: "Cheap attacks, reduced metabolism and long reach.",
      config: {
        attack_cost: 0.1,
        predator_metabolism_scale: 0.4,
        attack_range_bonus: 8,
      },
    },
    {
      id: "complex",
      name: "Complex habitat",
      description:
        "Intense predation with resource clusters and a central hazard; collapse is possible.",
      config: {
        attack_cost: 0.1,
        predator_metabolism_scale: 0.4,
        attack_range_bonus: 8,
        resource_zones: true,
        hazard_zones: true,
      },
    },
  ].map((preset): ExperimentPreset => ({
    ...preset,
    config: {
      predation: true,
      controller: "recurrent",
      initial_population: 1000,
      ...preset.config,
    },
  })),
];
