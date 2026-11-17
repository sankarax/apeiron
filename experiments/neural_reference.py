
import json
from dataclasses import dataclass
from pathlib import Path

import torch

from neural_inference import NeuralPopulation, RecurrentPopulation


DATA_DIR = Path(__file__).resolve().parent / "data"


@dataclass
class NeuralReference:
    network: NeuralPopulation
    inputs: torch.Tensor
    expected: torch.Tensor
    seed: int


def load_reference(
    path: Path,
    device: str = "cpu",
) -> NeuralReference:
    with path.open("r", encoding="utf-8") as file:
        data = json.load(file)

    architecture = tuple(data["architecture"])
    population = data["population"]

    network_type = RecurrentPopulation if "memory" in data else NeuralPopulation
    network = network_type(
        population,
        architecture,
        device=device,
        seed=data["seed"],
    )

    if isinstance(network, RecurrentPopulation):
        network.memory.copy_(torch.tensor(data["memory"], dtype=torch.float32, device=device))

    if len(data["layers"]) != len(network.layers):
        raise ValueError("layer count mismatch")

    for layer, reference in zip(network.layers, data["layers"]):
        weights = torch.tensor(
            reference["weights"],
            dtype=torch.float32,
            device=device,
        )

        biases = torch.tensor(
            reference["biases"],
            dtype=torch.float32,
            device=device,
        )

        if weights.shape != layer.weights.shape:
            raise ValueError("weight shape mismatch")

        if biases.shape != layer.biases.shape:
            raise ValueError("bias shape mismatch")

        layer.weights.copy_(weights)
        layer.biases.copy_(biases)

    inputs = torch.tensor(
        data["inputs"],
        dtype=torch.float32,
        device=device,
    )

    expected = torch.tensor(
        data["outputs"],
        dtype=torch.float32,
        device=device,
    )

    if inputs.shape != (population, architecture[0]):
        raise ValueError("input shape mismatch")

    output_size = 20 if isinstance(network, RecurrentPopulation) else architecture[-1]
    if expected.shape != (population, output_size):
        raise ValueError("output shape mismatch")

    return NeuralReference(
        network=network,
        inputs=inputs,
        expected=expected,
        seed=data["seed"],
    )


def compare(reference: NeuralReference):
    outputs = reference.network.forward(reference.inputs)

    error = (outputs - reference.expected).abs()

    return {
        "max_error": error.max().item(),
        "mean_error": error.mean().item(),
        "rmse": error.square().mean().sqrt().item(),
        "matches": torch.allclose(
            outputs,
            reference.expected,
            rtol=1e-5,
            atol=1e-6,
        ),
    }


def main():
    torch.set_float32_matmul_precision("highest")

    if not torch.cuda.is_available():
        raise RuntimeError("CUDA is required")

    print("Apeiron Rust-PyTorch numerical equivalence")

    for name in ("small", "deep", "recurrent"):
        path = DATA_DIR / f"{name}_reference.json"

        for device in ("cpu", "cuda"):
            reference = load_reference(path, device)
            result = compare(reference)

            print()
            print(f"Architecture: {reference.network.architecture}")
            print(f"Population: {reference.network.population:,}")
            print(f"Device: {device}")
            print(f"Seed: {reference.seed}")
            print(f"Max absolute error: {result['max_error']:.8e}")
            print(f"Mean absolute error: {result['mean_error']:.8e}")
            print(f"RMSE: {result['rmse']:.8e}")
            print(f"Matches Rust: {result['matches']}")

            if not result["matches"]:
                raise RuntimeError("numerical equivalence failed")


if __name__ == "__main__":
    main()
