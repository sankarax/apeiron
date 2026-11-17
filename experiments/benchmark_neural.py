
import statistics
import time

import torch

from neural_inference import (
    NeuralPopulation,
    RecurrentPopulation,
    RECURRENT_ARCHITECTURE,
    SMALL_ARCHITECTURE,
    DEEP_ARCHITECTURE,
)


POPULATIONS = (100, 1000, 5000, 10000, 20000, 50000, 100000)
WARMUP = 30
ITERATIONS = 100
TRIALS = 5


def synchronize(device):
    if device.type == "cuda":
        torch.cuda.synchronize(device)


def benchmark(network, inputs):
    device = network.device

    for _ in range(WARMUP):
        network.forward(inputs)

    synchronize(device)

    timings = []

    for _ in range(TRIALS):
        synchronize(device)
        start = time.perf_counter()

        for _ in range(ITERATIONS):
            network.forward(inputs)

        synchronize(device)

        elapsed = time.perf_counter() - start
        timings.append(elapsed / ITERATIONS)

    return statistics.median(timings)


def copy_parameters(source, target):
    for src, dst in zip(source.layers, target.layers):
        dst.weights.copy_(src.weights)
        dst.biases.copy_(src.biases)


def run_case(population, architecture):
    network_type = RecurrentPopulation if architecture == RECURRENT_ARCHITECTURE else NeuralPopulation
    cpu = network_type(
        population,
        architecture,
        device="cpu",
    )

    gpu = network_type(
        population,
        architecture,
        device="cuda",
    )

    copy_parameters(cpu, gpu)
    if isinstance(cpu, RecurrentPopulation):
        gpu.memory.copy_(cpu.memory)

    generator = torch.Generator().manual_seed(42)

    cpu_inputs = torch.rand(
        population,
        architecture[0],
        generator=generator,
    ) * 2.0 - 1.0

    gpu_inputs = cpu_inputs.cuda()

    cpu_outputs = cpu.forward(cpu_inputs)
    gpu_outputs = gpu.forward(gpu_inputs).cpu()

    torch.testing.assert_close(
        cpu_outputs,
        gpu_outputs,
        rtol=1e-5,
        atol=1e-6,
    )

    max_error = (cpu_outputs - gpu_outputs).abs().max().item()

    cpu_time = benchmark(cpu, cpu_inputs)

    torch.cuda.reset_peak_memory_stats()

    gpu_time = benchmark(gpu, gpu_inputs)

    peak_memory = torch.cuda.max_memory_allocated() / 1e6

    return {
        "population": population,
        "cpu_ms": cpu_time * 1000,
        "gpu_ms": gpu_time * 1000,
        "speedup": cpu_time / gpu_time,
        "cpu_throughput": population / cpu_time,
        "gpu_throughput": population / gpu_time,
        "parameter_mb": gpu.parameter_memory_bytes / 1e6,
        "gpu_peak_mb": peak_memory,
        "max_error": max_error,
    }


def main():
    if not torch.cuda.is_available():
        raise RuntimeError("CUDA is required for this benchmark")

    torch.set_float32_matmul_precision("highest")

    print("Apeiron neural inference benchmark")
    print(f"PyTorch: {torch.__version__}")
    print(f"CUDA: {torch.version.cuda}")
    print(f"GPU: {torch.cuda.get_device_name(0)}")
    print(f"CPU threads: {torch.get_num_threads()}")
    print(f"Warmup: {WARMUP}")
    print(f"Iterations: {ITERATIONS}")
    print(f"Trials: {TRIALS}")
    print("Precision: FP32")
    print("Timing: median synchronized wall-clock latency")

    for name, architecture in (
        ("Small MLP", SMALL_ARCHITECTURE),
        ("Deep MLP", DEEP_ARCHITECTURE),
        ("GRU (output and next memory)", RECURRENT_ARCHITECTURE),
    ):
        print()
        print(f"{name}: {architecture}")
        print("-" * 125)

        print(
            f"{'Population':>12}"
            f"{'CPU ms':>12}"
            f"{'GPU ms':>12}"
            f"{'Speedup':>10}"
            f"{'CPU org/s':>15}"
            f"{'GPU org/s':>15}"
            f"{'Params MB':>12}"
            f"{'Peak MB':>12}"
            f"{'Max error':>15}"
        )

        for population in POPULATIONS:
            result = run_case(population, architecture)

            print(
                f"{population:>12,}"
                f"{result['cpu_ms']:>12.4f}"
                f"{result['gpu_ms']:>12.4f}"
                f"{result['speedup']:>9.2f}x"
                f"{result['cpu_throughput']:>15,.0f}"
                f"{result['gpu_throughput']:>15,.0f}"
                f"{result['parameter_mb']:>12.2f}"
                f"{result['gpu_peak_mb']:>12.2f}"
                f"{result['max_error']:>15.2e}",
                flush=True,
            )

    print()
    print("Benchmark complete")


if __name__ == "__main__":
    main()
