
import statistics
import time

import torch

from neural_inference import (
    NeuralPopulation,
    SMALL_ARCHITECTURE,
    DEEP_ARCHITECTURE,
)


POPULATIONS = (100, 1000, 5000, 10000, 20000, 50000, 100000)
WARMUP = 30
ITERATIONS = 100
TRIALS = 5


def benchmark(operation, synchronize=False):
    for _ in range(WARMUP):
        operation()

        if synchronize:
            torch.cuda.synchronize()

    torch.cuda.synchronize()

    timings = []

    for _ in range(TRIALS):
        torch.cuda.synchronize()
        start = time.perf_counter()

        for _ in range(ITERATIONS):
            operation()

            if synchronize:
                torch.cuda.synchronize()

        torch.cuda.synchronize()

        elapsed = time.perf_counter() - start
        timings.append(elapsed / ITERATIONS * 1000)

    return statistics.median(timings)


def run_case(population, architecture):
    network = NeuralPopulation(
        population,
        architecture,
        device="cuda",
    )

    cpu_inputs = torch.rand(
        population,
        architecture[0],
    ) * 2.0 - 1.0

    pinned_inputs = cpu_inputs.pin_memory()

    gpu_inputs = torch.empty_like(
        cpu_inputs,
        device="cuda",
    )

    gpu_inputs.copy_(cpu_inputs)

    gpu_outputs = network.forward(gpu_inputs)

    cpu_outputs = torch.empty(
        population,
        architecture[-1],
        dtype=torch.float32,
    )

    pinned_outputs = torch.empty_like(
        cpu_outputs,
        pin_memory=True,
    )

    def resident():
        network.forward(gpu_inputs)

    def upload():
        gpu_inputs.copy_(cpu_inputs)

    def download():
        cpu_outputs.copy_(gpu_outputs)

    def round_trip():
        gpu_inputs.copy_(cpu_inputs)
        outputs = network.forward(gpu_inputs)
        cpu_outputs.copy_(outputs)

    def pinned_round_trip():
        gpu_inputs.copy_(
            pinned_inputs,
            non_blocking=True,
        )

        outputs = network.forward(gpu_inputs)

        pinned_outputs.copy_(
            outputs,
            non_blocking=True,
        )

    resident_ms = benchmark(resident, synchronize=True)
    upload_ms = benchmark(upload, synchronize=True)
    download_ms = benchmark(download, synchronize=True)
    round_trip_ms = benchmark(round_trip, synchronize=True)
    pinned_ms = benchmark(pinned_round_trip, synchronize=True)

    expected = network.forward(gpu_inputs).cpu()

    torch.testing.assert_close(cpu_outputs, expected)
    torch.testing.assert_close(pinned_outputs, expected)

    transfer_mb = (
        cpu_inputs.numel() * cpu_inputs.element_size()
        + cpu_outputs.numel() * cpu_outputs.element_size()
    ) / 1e6

    return {
        "population": population,
        "resident_ms": resident_ms,
        "upload_ms": upload_ms,
        "download_ms": download_ms,
        "round_trip_ms": round_trip_ms,
        "pinned_ms": pinned_ms,
        "transfer_mb": transfer_mb,
        "overhead_ms": round_trip_ms - resident_ms,
        "pinned_speedup": round_trip_ms / pinned_ms,
    }


def main():
    if not torch.cuda.is_available():
        raise RuntimeError("CUDA is required for this benchmark")

    torch.set_float32_matmul_precision("highest")

    print("Apeiron neural transfer benchmark")
    print(f"PyTorch: {torch.__version__}")
    print(f"CUDA: {torch.version.cuda}")
    print(f"GPU: {torch.cuda.get_device_name(0)}")
    print(f"Warmup: {WARMUP}")
    print(f"Iterations: {ITERATIONS}")
    print(f"Trials: {TRIALS}")
    print("Timing: synchronized latency per operation")
    print("Precision: FP32")

    for name, architecture in (
        ("Small MLP", SMALL_ARCHITECTURE),
        ("Deep MLP", DEEP_ARCHITECTURE),
    ):
        print()
        print(f"{name}: {architecture}")
        print("-" * 126)

        print(
            f"{'Population':>12}"
            f"{'Resident ms':>14}"
            f"{'H2D ms':>12}"
            f"{'D2H ms':>12}"
            f"{'Roundtrip ms':>15}"
            f"{'Pinned ms':>12}"
            f"{'Overhead ms':>14}"
            f"{'Pinned gain':>13}"
            f"{'Data MB':>10}"
        )

        for population in POPULATIONS:
            result = run_case(population, architecture)

            print(
                f"{population:>12,}"
                f"{result['resident_ms']:>14.4f}"
                f"{result['upload_ms']:>12.4f}"
                f"{result['download_ms']:>12.4f}"
                f"{result['round_trip_ms']:>15.4f}"
                f"{result['pinned_ms']:>12.4f}"
                f"{result['overhead_ms']:>14.4f}"
                f"{result['pinned_speedup']:>12.2f}x"
                f"{result['transfer_mb']:>10.3f}",
                flush=True,
            )

    print()
    print("Benchmark complete")


if __name__ == "__main__":
    main()
