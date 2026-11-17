# Engineering decisions

## Rust core and Python service

Rust provides explicit ownership, predictable data layout and native parallel execution. Python remains useful for experiment orchestration, API development and PyTorch validation. PyO3 avoids rewriting the simulation in Python. JSON snapshots simplify integration but add allocation and serialization overhead.

## Spatial grids and ordered updates

A uniform grid is simpler than a tree for a bounded, moving two-dimensional world. Its effectiveness depends on density. Brute-force search remains a reference for correctness. Parallelizing independent calculations while retaining ordered shared-resource updates avoids races and preserves deterministic random draws. The cost is a serial portion of every tick.

## CPU and GPU

Rayon has scheduling overhead at small populations. Auto CPU uses a threshold of 500 organisms, based on the measured crossover. GPU execution is explicit rather than automatically selected because the crossover depends on controller, population and predation. Native GPU sensing and inference are retained even though GPU execution is slower in some workloads.

GPU development progressed from reusable buffers to resident weights, batched submissions and stable slots. Smaller tiles reduce the amount of data rewritten for births and deaths. The 16-organism tile improved predation workloads relative to the 64-organism tile, but some non-predation results regressed relative to the original profile. See the full comparison in [performance](performance.md).

## Browser deployment

WASM reuses the Rust engine without requiring a hosted backend. Web Workers isolate simulation work from UI rendering. Browser WebGPU was tested and rejected for the release because synchronous tick dependencies and transfer overhead made the observed implementation slower than browser CPU execution. This does not establish that WebGPU is inherently slower. Native GPU support remains available.

## Persistence and research

PostgreSQL provides durable records and transaction semantics. Redis is an optional cache for immutable research payloads, not a second source of truth. A local Ollama model can call bounded experiment tools without an external API subscription. Tool outputs are evidence; generated interpretations can be wrong. Public research uses recorded data rather than a live model.

## Limits and future work

One million organisms is not a validated target. Recurrent parameters alone would occupy about 6.6 GB in decimal units, before memory, metadata, spatial indices and GPU buffers. A larger system would need profiling at representative densities, more compact layouts, bounded history, reduced snapshot bandwidth and possibly GPU-resident ecology or partitioned execution. Parallel feeding and predation would need explicit conflict resolution. None of these changes is implemented or benchmarked at that scale.

Other limitations include finite experimental horizons, no controlled memory-disabled recurrent ablation, no cross-device exact GPU replay guarantee, and no production authentication or distributed job scheduling.
