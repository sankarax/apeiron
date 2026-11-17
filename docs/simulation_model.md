# Simulation model

The environment is a continuous rectangular world with periodic boundaries. Positions wrap at the edges. Distance calculations use the shortest displacement on the torus, for example `((to - from + L/2) mod L) - L/2` with Euclidean modulo.

## Defaults

| Setting | Value |
| --- | ---: |
| Width / height | 1600 / 900 |
| Initial / maximum population | 200 / 5000 |
| Initial / maximum food | 500 / 2000 |
| Food spawn per tick | 10 |
| Food energy | 25 |
| Initial / maximum organism energy | 100 / 250 |
| Maximum age | 3000 ticks |
| Trait mutation probability / strength | 0.1 / 0.15 |
| Seed | 42 |

Source: `engine/src/config.rs`. Predation, resource zones and hazards are disabled by default.

## Organisms

An organism has an ID, parent ID, generation, position, velocity, age, energy, inherited traits and controller. Traits include speed, sensor radius, metabolism, size, reproduction threshold and hue. Energy is reduced by basal metabolism and costs associated with movement, sensing and size. The basic movement cost includes `0.02 * (vx² + vy²)`, sensing `0.001 * sensor_radius` and size `0.01 * size`. Predator scaling and hazards modify the energy budget.

Food consumption increases energy up to the configured maximum. Feeding is resolved in order so a food item cannot be consumed twice. Organisms die when energy is exhausted or the age limit is reached.

## Evolution

Eligible organisms reproduce asexually. The parent transfers 40% of its energy to the child, retaining 60%. The child inherits traits and controller parameters with random bounded mutations. Recurrent memory starts at zero in offspring. Reproduction depends on age, energy and population capacity. There is no explicit fitness score, reward optimization or backpropagation. Differential survival and reproduction provide selection pressure.

The population accounting invariant is `initial population + births - deaths = current population`. Reproduction conserves parent-child energy before other tick costs. The environment is not a closed energy system because food is generated and energy is dissipated.

## Predation

Deep and recurrent controllers support aggression. The initial predator fraction defaults to 0.2; species is inherited. Predators do not consume plants. Attacks depend on proximity and aggression, cost energy and transfer a configurable fraction of prey energy. Defaults are attack cost 1, range bonus 2 and efficiency 0.7. Predation is resolved in order to prevent duplicate kills. Extinction is allowed; predators are not automatically replenished.

Resource zones concentrate food and hazard zones drain energy. These are configurable experimental pressures, not a validated biological ecosystem model. Hazard-specific neural sensing is not implemented.

## Spatial indexing

Uniform grids group food and organisms into cells. Queries visit nearby cells, including wrapped neighbors, then check exact distances. Large-radius queries avoid duplicate cells. This reduces candidate comparisons when density is bounded, but dense cells can still produce expensive queries. Sensing and edible-food selection retain their respective boundary and tie-breaking rules; brute-force comparisons are included in tests.

## Metrics and reproducibility

Metrics include population, food, births, deaths, energy, trait means, generation and predation counters. A seed initializes the random generator. Ordered random draws and resource resolution preserve CPU reproducibility across supported backends. A checkpoint additionally saves the current RNG position and controller state. A seed alone is insufficient to resume a simulation midway.
