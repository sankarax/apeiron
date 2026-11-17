# Architecture

## Execution paths

```text
Native
React / Three.js -> HTTP + WebSocket -> FastAPI -> PyO3 -> Rust
                                      |                  |
                                      |                  +-> Rayon / wgpu
                                      +-> PostgreSQL / Redis
                                      +-> Ollama

Browser
React / Three.js -> worker messages -> Web Worker -> Rust WASM
```

Both editions use the same Rust simulation model. The browser build selects serial CPU execution and keeps simulation work outside the main UI thread. Each browser multiverse world has its own worker. The native Python binding exposes the simulation through `PySimulation`; native GPU selection is handled separately from the four CPU backend variants.

## Engine

| Module | Responsibility |
| --- | --- |
| `simulation.rs`, `world.rs` | Tick orchestration and world state |
| `organism.rs`, `genome.rs` | Organism state, movement, reproduction and traits |
| `spatial.rs`, `spatial_grid.rs`, `food_search.rs` | Wrapped distances and local queries |
| `sensors.rs`, `sensory_pipeline.rs` | Input features |
| `neural.rs`, `recurrent.rs`, `neural_genome.rs` | Controller inference and inheritance |
| `neural_mutation.rs` | Parameter mutation |
| `parallel.rs`, `compute_backend.rs` | CPU execution selection |
| `gpu/` | Native compute shaders and buffer management |
| `checkpoint.rs`, `lineage.rs` | State restoration and ancestry |
| `python.rs`, `browser.rs` | Runtime bindings |
| `metrics.rs`, `profiling.rs` | Statistics and timing |

A tick spawns food, builds spatial indices, senses and moves organisms, resolves feeding and predation, reproduces eligible organisms and removes dead organisms. Independent sensing and inference can be parallelized; shared resource consumption and reproduction require ordered updates. Native GPU execution accelerates sensing and neural inference, not the entire ecology pipeline.

## Native service

`backend/app/simulation_manager.py` owns the live simulation and uses an asyncio lock to serialize changes. Rust work is dispatched through a worker thread and the Python binding releases the interpreter during computation. The simulation loop batches ticks according to the target rate. WebSocket snapshots are sent independently of tick execution; rendering frequency is not simulation throughput.

REST routes provide configuration, start, pause, step, reset, metrics and brain inspection. `/ws/simulation` streams state. The backend selector uses `/simulation/backend` (without the `/api` prefix). See `main.py` and the generated FastAPI `/docs` page for the exact request schemas. The multiverse, history, scientist and archive routers provide separate workflows.

## Frontend

React manages controls and panels; Zustand stores the latest simulation snapshot and sampled history. Three.js renders organisms and food using instanced geometry. Interpolation smooths movement between snapshots and respects wrapped boundaries. uPlot renders sampled time series. Neural inspection is requested for the selected organism rather than serializing every network in each snapshot.

Browser mode is selected by Vite's `browser` mode. The service layer translates requests into worker messages. The worker owns the WASM simulation and returns snapshots and inspection data. Browser worlds are temporary and are not backed by PostgreSQL.

## Persistence

PostgreSQL stores checkpoint and research records in `apeiron_records`, with a text identifier, kind, name, optional tick, JSONB payload and creation time. A transaction-scoped advisory lock enforces the eight-checkpoint limit. Redis caches immutable research payloads for 300 seconds; PostgreSQL remains authoritative. Cache errors fall back to database reads. The native backend requires PostgreSQL at startup.

Checkpoints contain state not present in ordinary snapshots, including neural parameters, recurrent memory and RNG position. CPU replay has been checked for deterministic continuation. Exact GPU replay across devices is not established.

## Boundaries

The local API is intended for loopback access. It has no user authentication or multi-tenant isolation. The public edition does not expose the native backend. Neither edition is a distributed simulation system.
