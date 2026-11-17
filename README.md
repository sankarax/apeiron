# Apeiron
[Live Demo](https://apeiron-ehr.pages.dev/)

Apeiron is an artificial life simulation engine written in Rust. Organisms move, consume resources, reproduce, mutate and die in a two-dimensional environment. Neural controllers evolve through inheritance and selection rather than gradient-based training.

The engine runs natively with CPU and GPU backends, or in a browser using WebAssembly. The React interface provides live simulation controls, population statistics, neural inspection and independent experimental worlds.

## Features

- Heuristic, small MLP, deep MLP and recurrent neural controllers
- Spatial indexing, multithreaded CPU execution and native wgpu compute shaders
- Configurable resource availability, predation and environmental hazards
- Organism ancestry, neural activations and population metrics
- Independent simulation worlds and batch execution
- Native checkpoints, deterministic CPU replay and persistent research records
- Local LLM-assisted experiments through Ollama

## Editions

| Feature | Browser | Native |
| --- | --- | --- |
| Simulation and neural inspection | Yes | Yes |
| Compute | Serial CPU in WebAssembly | Serial CPU, Rayon, wgpu |
| Independent worlds | Up to 4 | Up to 8 |
| Checkpoint storage and replay | No | Yes |
| Research | Recorded experiment | Live local model and archive |
| Server required | No | Yes |

## Setup

See [deployment](docs/deployment.md) for installation, build and test commands. The browser edition requires no backend, database or model server at runtime.

## Repository

```text
engine/       Rust simulation, Python and WASM bindings, tests
backend/      FastAPI, experiment orchestration and persistence
frontend/     React interface and browser worker
experiments/  PyTorch reference implementations and benchmarks
benchmarks/   Recorded benchmark outputs
scripts/      Build and local service scripts
docs/         Technical documentation
.github/      CI configuration
```

## Documentation

- [Architecture](docs/architecture.md)
- [Simulation model](docs/simulation_model.md)
- [Neural networks](docs/neural_networks.md)
- [Performance](docs/performance.md)
- [Engineering decisions](docs/engineering_decisions.md)
- [Research](docs/research.md)
- [Deployment and testing](docs/deployment.md)

## Status

Development benchmarks and numerical checks are recorded in `benchmarks/`. The latest cleanup passed 121 Rust library tests, Rust Clippy, WASM compilation and TypeScript compilation. The npm audit reported zero vulnerabilities after dependency updates. Final lint, integration tests and runtime regression checks remain pending. The public deployment and license selection are not yet complete.
