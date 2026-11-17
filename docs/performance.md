# Performance

## Methodology

Development measurements were collected on a Ryzen 9 laptop with 32 GB RAM and an NVIDIA RTX 4070 Laptop GPU (90 W configuration). Native CPU runs report 16 Rayon workers. PyTorch CUDA logs identify PyTorch 2.11.0+cu128 and CUDA 12.8. These are historical development measurements, not a fresh benchmark of the final release. Power, temperature and clock conditions were not systematically recorded. Different workloads and revisions must not be treated as identical.

TPS means simulation ticks per second, not rendering frames per second or individual neural inferences per second. A mean tick latency of t milliseconds corresponds to approximately 1000/t TPS. Inference-only measurements exclude ecology and, for resident CUDA inputs, host transfers. Raw files retain all samples reported by the original benchmark programs.

## Spatial and CPU optimization

The early 2000-tick ecosystem run improved from 3.743 s (534.29 TPS) using brute force to 2.054 s (973.85 TPS) with spatial sensing and 173.092 ms (11554.53 TPS) with spatial feeding as well. The recorded endpoint was 623 organisms, 116 food, 1726 births and 1303 deaths. These early timings were recorded during development; the later `post_neural_ecosystem.txt` run reports 334.562 ms (5977.96 TPS) with the same endpoint, a 48.3% throughput reduction from the early optimized run. It is not an isolated measurement of neural overhead.

The CPU crossover log shows parallel execution slower at 100, 200 and 300 organisms (0.50x, 0.71x, 0.93x), slightly faster at 400 (1.06x) and 1.38x at 500. The engine uses 500 as the automatic threshold. The early brute-force scaling log is not a fair fixed-food comparison because available food falls to zero at larger populations.

### Fixed world CPU scaling

| Population | Serial TPS | Parallel TPS | Auto TPS | Parallel / serial |
| ---: | ---: | ---: | ---: | ---: |
| 200 | 16642.53 | 8706.30 | 16242.41 | 0.52x |
| 1000 | 2262.10 | 4745.16 | 4546.19 | 2.10x |
| 5000 | 306.43 | 1097.98 | 1139.26 | 3.58x |
| 10000 | 79.42 | 380.99 | 369.24 | 4.80x |
| 20000 | 23.08 | 112.30 | 113.41 | 4.87x |
| 50000 | 3.97 | 22.14 | 21.77 | 5.58x |
| 100000 | 0.83 | 5.39 | 4.82 | 6.49x |

Source: [`final_cpu_scaling.txt`](../benchmarks/final_cpu_scaling.txt).

### Constant density CPU scaling

| Population | Serial TPS | Parallel TPS | Auto TPS | Parallel / serial |
| ---: | ---: | ---: | ---: | ---: |
| 1000 | 2280.07 | 4208.45 | 4380.70 | 1.85x |
| 5000 | 406.91 | 1153.16 | 1127.93 | 2.83x |
| 10000 | 199.38 | 623.74 | 648.88 | 3.13x |
| 20000 | 89.75 | 285.72 | 281.39 | 3.18x |
| 50000 | 20.53 | 61.39 | 63.20 | 2.99x |
| 100000 | 8.61 | 26.49 | 26.17 | 3.08x |

Source: [`constant_density.txt`](../benchmarks/constant_density.txt).

The fixed-world run uses a 1600 by 900 world, five warmup ticks and three trials (median). The constant-density run increases world area with population and uses one food item per organism initially. Density changes candidate counts and makes the two workloads distinct. At 100,000 organisms auto CPU achieves 4.82 TPS in the fixed world and 26.17 TPS at constant density, not 60 TPS.

## Neural inference

At 100,000 independent networks the final Rust serial/parallel latencies are 7.9407/0.9637 ms (small), 46.7208/10.8455 ms (deep), and 56.9281/15.7403 ms (recurrent). Resident PyTorch CUDA latencies are 0.4517, 2.3105 and 3.4819 ms respectively. CUDA is approximately 2.13x, 4.69x and 4.52x faster than Rayon for these inference-only cases. See `final_rust_neural.txt` and `final_pytorch_neural.txt` for every population and trial summary. The transfer benchmark separately measures host-to-device, device-to-host, roundtrip and pinned memory latency; these costs are not included in resident inference throughput.

The final Rust/PyTorch comparison reports maximum absolute errors below 4e-7 for the six CPU/CUDA cases. Earlier native GPU checks compared 27,000 neural values and 96,000 sensor values; maximum absolute errors were approximately 3.28e-7 and 1.79e-7 respectively. These are tolerance checks, not a proof of identical long-term trajectories.

## Reproduction

Boxing and copying changes reduced clone latency but did not consistently improve mutation throughput. The table compares every recorded reproduction case. Positive change means more offspring per second; negative means a regression.

| Controller | Population | Original clone ms | Later clone ms | Original mutation ms | Later mutation ms | Offspring/s change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Heuristic | 100 | 0.0059 | 0.0061 | 0.0055 | 0.0061 | -9.6% |
| Heuristic | 1000 | 0.0579 | 0.0611 | 0.0576 | 0.0617 | -6.6% |
| Heuristic | 5000 | 0.2971 | 0.3531 | 0.3052 | 0.3538 | -13.7% |
| Heuristic | 10000 | 0.6062 | 0.7065 | 0.5976 | 0.6873 | -13.1% |
| Heuristic | 20000 | 1.5243 | 1.6535 | 1.5265 | 1.6738 | -8.8% |
| Small | 100 | 0.0595 | 0.0338 | 0.0687 | 0.0730 | -5.9% |
| Small | 1000 | 0.5211 | 0.1949 | 0.6232 | 0.6702 | -7.0% |
| Small | 5000 | 2.8567 | 1.1812 | 3.5834 | 3.7678 | -4.9% |
| Small | 10000 | 5.6043 | 4.4691 | 6.7056 | 7.4569 | -10.1% |
| Small | 20000 | 12.1920 | 5.8606 | 14.4738 | 15.4213 | -6.1% |
| Deep | 100 | 0.6028 | 0.4105 | 0.6623 | 0.9836 | -32.7% |
| Deep | 1000 | 6.4259 | 2.7944 | 7.9906 | 8.5148 | -6.2% |
| Deep | 5000 | 31.4849 | 20.0883 | 38.8929 | 42.6012 | -8.7% |
| Deep | 10000 | 63.3436 | 22.9140 | 85.5942 | 90.3223 | -5.2% |
| Deep | 20000 | 130.4998 | 34.2452 | 184.8736 | 185.1298 | -0.1% |

Sources: `organism_reproduction.txt`, `optimized_reproduction.txt`. Both report three warmups, ten iterations and five trials. Differences between runs are not controlled causal estimates.

## Native GPU scaling

The first three GPU scaling logs compare reusable buffers, resident weights and batched submissions. Each cell is mean GPU milliseconds per tick. The last column is the change in throughput from the first to the last run.

| Controller | Population | Initial | Resident | Batched | TPS change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Small | 1000 | 0.649 | 0.744 | 0.524 | +23.9% |
| Small | 5000 | 2.207 | 1.561 | 1.130 | +95.3% |
| Small | 10000 | 3.846 | 3.004 | 1.884 | +104.1% |
| Small | 20000 | 8.810 | 5.601 | 4.243 | +107.6% |
| Deep | 1000 | 2.268 | 1.214 | 1.068 | +112.4% |
| Deep | 5000 | 10.375 | 3.996 | 4.039 | +156.9% |
| Deep | 10000 | 25.119 | 9.313 | 9.169 | +174.0% |
| Deep | 20000 | 45.544 | 15.611 | 14.364 | +217.1% |
| Recurrent | 1000 | 3.082 | 1.078 | 1.059 | +191.0% |
| Recurrent | 5000 | 16.282 | 2.901 | 2.786 | +484.4% |
| Recurrent | 10000 | 31.806 | 8.820 | 8.873 | +258.5% |
| Recurrent | 20000 | 58.198 | 19.091 | 17.491 | +232.7% |

The batched run shows GPU crossover between 10,000 and 20,000 for deep and recurrent controllers; small remains slower than CPU throughout this particular scaling run. Theoretical unchanged-tick recurrent transfer volume at 20,000 organisms falls from 134.72 MB to 2.56 MB when weights remain resident (98.1% reduction); this is a byte-count estimate, not measured PCIe bandwidth.

## Predation profiling and stable slots

The later profiling suite reports mean tick times after five warmup ticks and 30 measured ticks. Movement includes sensing and inference. Stable slots and 64-organism tiles were introduced to reduce weight rewrites during births and deaths, followed by 16-organism tiles. The table includes CPU and GPU results for every workload, including regressions.

| Controller | Predation | Initial population | Backend | Original ms | Tile 64 ms | Tile 16 ms | TPS change original to final | TPS change tile 64 to 16 |
| --- | --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| Small | false | 1000 | cpu | 0.1971 | 0.1792 | 0.2011 | -2.0% | -10.9% |
| Small | false | 1000 | gpu | 0.5671 | 0.6524 | 0.7439 | -23.8% | -12.3% |
| Small | false | 5000 | cpu | 1.1154 | 1.1450 | 1.0026 | +11.3% | +14.2% |
| Small | false | 5000 | gpu | 1.3972 | 1.5283 | 1.9094 | -26.8% | -20.0% |
| Small | false | 20000 | cpu | 10.8629 | 11.0526 | 12.0106 | -9.6% | -8.0% |
| Small | false | 20000 | gpu | 7.0307 | 8.8043 | 8.3925 | -16.2% | +4.9% |
| Deep | false | 1000 | cpu | 0.2838 | 0.3540 | 0.3023 | -6.1% | +17.1% |
| Deep | false | 1000 | gpu | 1.2096 | 1.0838 | 1.1000 | +10.0% | -1.5% |
| Deep | false | 5000 | cpu | 3.2561 | 2.6223 | 2.6372 | +23.5% | -0.6% |
| Deep | false | 5000 | gpu | 4.5823 | 4.1643 | 4.2181 | +8.6% | -1.3% |
| Deep | false | 20000 | cpu | 31.1772 | 29.0710 | 29.4795 | +5.8% | -1.4% |
| Deep | false | 20000 | gpu | 19.1206 | 15.5816 | 14.9532 | +27.9% | +4.2% |
| Recurrent | false | 1000 | cpu | 0.2909 | 0.3386 | 0.3240 | -10.2% | +4.5% |
| Recurrent | false | 1000 | gpu | 1.1967 | 1.1197 | 1.1200 | +6.8% | -0.0% |
| Recurrent | false | 5000 | cpu | 3.1668 | 2.7876 | 2.7727 | +14.2% | +0.5% |
| Recurrent | false | 5000 | gpu | 3.2560 | 3.6833 | 3.0994 | +5.1% | +18.8% |
| Recurrent | false | 20000 | cpu | 31.0877 | 30.5076 | 30.0871 | +3.3% | +1.4% |
| Recurrent | false | 20000 | gpu | 21.6746 | 16.2111 | 15.8885 | +36.4% | +2.0% |
| Recurrent | true | 1000 | cpu | 0.3386 | 0.4025 | 0.3291 | +2.9% | +22.3% |
| Recurrent | true | 1000 | gpu | 4.7990 | 1.6911 | 1.4558 | +229.6% | +16.2% |
| Recurrent | true | 5000 | cpu | 3.3411 | 3.0672 | 3.3120 | +0.9% | -7.4% |
| Recurrent | true | 5000 | gpu | 35.8731 | 8.6029 | 7.7258 | +364.3% | +11.4% |
| Recurrent | true | 20000 | cpu | 22.7694 | 21.4477 | 21.1694 | +7.6% | +1.3% |
| Recurrent | true | 20000 | gpu | 254.2049 | 51.5210 | 49.2806 | +415.8% | +4.5% |

At 20,000 initial recurrent organisms with predation, GPU latency fell from 254.2049 to 49.2806 ms (5.16x throughput, 80.61% latency reduction). CPU remained faster at 21.1694 ms. GPU movement still accounted for 44.1747 ms of the final tick. Without predation at 20,000 organisms, the final GPU is 1.43x, 1.97x and 1.89x faster than the corresponding CPU for small, deep and recurrent. Relative to the original GPU profile, small throughput fell about 16.2%, while deep and recurrent rose about 27.9% and 36.4%. Final populations and kill counts agree across these profile runs; that does not establish equality of every state variable.

## Ecosystem experiments and regressions

The original world-frame small neural controller went extinct in all three recorded seeds. Egocentric sensing changed the result to survival in all three. The recurrent experiment reports survival for all twelve controller/seed runs at 10,000 ticks; recurrent final populations were 1404, 1464 and 1491. Recurrent mean throughput was 1241.87 TPS versus deep 1737.28 TPS. Compared with the earlier egocentric experiment, the heuristic, small and deep throughput measurements decreased by approximately 25.2%, 21.1% and 14.5%. These separate runs do not identify the cause. The capped final CPU scaling run reports auto CPU at 20,000 organisms: heuristic 83.06 TPS, small 91.66, deep 30.74 and recurrent 30.29. See [research](research.md) for ecology outcomes.

## Browser WebGPU experiment

During development, a browser GPU implementation dispatched neural inference and read back actions each tick. A subsequent version reused buffers and bind groups but still uploaded weights and synchronized each tick. At initial population 1000, recurrent controller, seed 42, predation disabled and target 500 TPS, Firefox dashboard readings were about 495-500 TPS for CPU and 10 TPS for GPU. Edge GPU readings were about 80-140 TPS. These were interactive dashboard observations, not controlled benchmarks; Edge CPU throughput was not measured in that comparison. Earlier tests at target 60 TPS were rate-limited and cannot establish maximum throughput. Browser GPU was removed from the release; native GPU was retained.

## Raw benchmark index

All 33 original logs remain in `benchmarks/`. They include historical versions and may not be reproducible with the current source alone. The links below preserve the full recorded results, including intermediate measurements and regressions. Some logs use UTF-16 encoding.

- [constant_density.txt](../benchmarks/constant_density.txt)
- [cpu_crossover.txt](../benchmarks/cpu_crossover.txt)
- [ecology_experiments.txt](../benchmarks/ecology_experiments.txt)
- [feeding_grid_profiling.txt](../benchmarks/feeding_grid_profiling.txt)
- [final_cpu_scaling.txt](../benchmarks/final_cpu_scaling.txt)
- [final_neural_equivalence.txt](../benchmarks/final_neural_equivalence.txt)
- [final_neural_scaling.txt](../benchmarks/final_neural_scaling.txt)
- [final_neural_scaling_capped.txt](../benchmarks/final_neural_scaling_capped.txt)
- [final_pytorch_neural.txt](../benchmarks/final_pytorch_neural.txt)
- [final_rust_neural.txt](../benchmarks/final_rust_neural.txt)
- [gpu_scaling_batched.txt](../benchmarks/gpu_scaling_batched.txt)
- [gpu_scaling_initial.txt](../benchmarks/gpu_scaling_initial.txt)
- [gpu_scaling_resident.txt](../benchmarks/gpu_scaling_resident.txt)
- [neural_ecosystem.txt](../benchmarks/neural_ecosystem.txt)
- [neural_ecosystem_egocentric.txt](../benchmarks/neural_ecosystem_egocentric.txt)
- [neural_equivalence.txt](../benchmarks/neural_equivalence.txt)
- [neural_inheritance.txt](../benchmarks/neural_inheritance.txt)
- [neural_memory.txt](../benchmarks/neural_memory.txt)
- [neural_transfers.txt](../benchmarks/neural_transfers.txt)
- [optimized_reproduction.txt](../benchmarks/optimized_reproduction.txt)
- [organism_reproduction.txt](../benchmarks/organism_reproduction.txt)
- [parallel_movement.txt](../benchmarks/parallel_movement.txt)
- [parallel_sensing.txt](../benchmarks/parallel_sensing.txt)
- [performance_profile_initial.txt](../benchmarks/performance_profile_initial.txt)
- [performance_profile_resident_slots.txt](../benchmarks/performance_profile_resident_slots.txt)
- [performance_profile_tile16.txt](../benchmarks/performance_profile_tile16.txt)
- [pipeline_profiling.txt](../benchmarks/pipeline_profiling.txt)
- [post_neural_ecosystem.txt](../benchmarks/post_neural_ecosystem.txt)
- [post_neural_scaling.txt](../benchmarks/post_neural_scaling.txt)
- [predation_initial.txt](../benchmarks/predation_initial.txt)
- [pytorch_inference.txt](../benchmarks/pytorch_inference.txt)
- [recurrent_ecosystem.txt](../benchmarks/recurrent_ecosystem.txt)
- [rust_neural_inference.txt](../benchmarks/rust_neural_inference.txt)
- [serial_cpu_baseline.txt](../benchmarks/serial_cpu_baseline.txt)
- [simulation_scaling.txt](../benchmarks/simulation_scaling.txt)

## Reproduction commands

Run from `engine/` in a release build. The current examples reproduce the current implementation, not historical revisions.

```powershell
cargo run --release --example crossover_benchmark
cargo run --release --example scaling_benchmark
cargo run --release --example density_benchmark
cargo run --release --example benchmark_neural
cargo run --release --example benchmark_reproduction
cargo run --release --example gpu_neural
cargo run --release --example gpu_sensing
cargo run --release --example gpu_simulation
cargo run --release --example gpu_scaling
cargo run --release --example performance_profile
cargo run --release --example ecology
```

For future measurements, record commit, configuration, seed, hardware, power mode, warmup, trial count, population trajectory and timing distribution. Compare matched workloads and report failures alongside improvements.
