
import math
from dataclasses import dataclass

import torch


SMALL_ARCHITECTURE = (6, 8, 3)
DEEP_ARCHITECTURE = (16, 32, 16, 4)
RECURRENT_ARCHITECTURE = (16, 16, 4)


@dataclass
class BatchedDense:
    weights: torch.Tensor
    biases: torch.Tensor

    @classmethod
    def random(
        cls,
        population: int,
        in_features: int,
        out_features: int,
        device: torch.device,
        generator: torch.Generator,
    ) -> "BatchedDense":
        limit = math.sqrt(6.0 / (in_features + out_features))

        weights = torch.empty(
            population,
            out_features,
            in_features,
            device=device,
            dtype=torch.float32,
        )

        weights.uniform_(-limit, limit, generator=generator)

        biases = torch.zeros(
            population,
            out_features,
            device=device,
            dtype=torch.float32,
        )

        return cls(weights, biases)

    def forward(self, inputs: torch.Tensor) -> torch.Tensor:
        return torch.bmm(
            self.weights,
            inputs.unsqueeze(-1),
        ).squeeze(-1) + self.biases

    @property
    def parameter_count(self) -> int:
        return self.weights.numel() + self.biases.numel()


class NeuralPopulation:
    def __init__(
        self,
        population: int,
        architecture: tuple[int, ...] = DEEP_ARCHITECTURE,
        device: str = "cpu",
        seed: int = 42,
    ):
        if population <= 0:
            raise ValueError("population must be positive")

        if len(architecture) < 2 or any(n <= 0 for n in architecture):
            raise ValueError("invalid network architecture")

        self.population = population
        self.architecture = architecture
        self.device = torch.device(device)

        generator = torch.Generator(device=self.device)
        generator.manual_seed(seed)

        self.layers = [
            BatchedDense.random(
                population,
                in_features,
                out_features,
                self.device,
                generator,
            )
            for in_features, out_features in zip(
                architecture[:-1],
                architecture[1:],
            )
        ]

    @torch.inference_mode()
    def forward(self, inputs: torch.Tensor) -> torch.Tensor:
        expected_shape = (self.population, self.architecture[0])

        if inputs.shape != expected_shape:
            raise ValueError(f"expected input shape {expected_shape}")

        for layer in self.layers:
            inputs = torch.tanh(layer.forward(inputs))

        return inputs

    @property
    def parameter_count(self) -> int:
        return sum(layer.parameter_count for layer in self.layers)

    @property
    def parameters_per_organism(self) -> int:
        return self.parameter_count // self.population

    @property
    def parameter_memory_bytes(self) -> int:
        return sum(
            layer.weights.numel() * layer.weights.element_size()
            + layer.biases.numel() * layer.biases.element_size()
            for layer in self.layers
        )


class RecurrentPopulation(NeuralPopulation):
    def __init__(self, population, architecture=RECURRENT_ARCHITECTURE, device="cpu", seed=42):
        if population <= 0 or architecture != RECURRENT_ARCHITECTURE:
            raise ValueError("invalid recurrent population")
        self.population = population
        self.architecture = architecture
        self.device = torch.device(device)
        generator = torch.Generator(device=self.device).manual_seed(seed)
        self.layers = [
            BatchedDense.random(population, n, m, self.device, generator)
            for n, m in [(32, 16)] * 3 + [(16, 4)]
        ]
        self.memory = torch.rand(population, 16, device=self.device, generator=generator) * 2 - 1

    @torch.inference_mode()
    def forward(self, inputs):
        if inputs.shape != (self.population, 16):
            raise ValueError("invalid input shape")
        combined = torch.cat((inputs, self.memory), dim=1)
        update = self.layers[0].forward(combined).sigmoid()
        reset = self.layers[1].forward(combined).sigmoid()
        candidate = self.layers[2].forward(
            torch.cat((inputs, reset * self.memory), dim=1)
        ).tanh()
        memory = (1 - update) * self.memory + update * candidate
        output = self.layers[3].forward(memory).tanh()
        return torch.cat((output, memory), dim=1)


def main():
    device = "cuda" if torch.cuda.is_available() else "cpu"
    population = 10000

    print(f"Device: {device}")

    for architecture in (SMALL_ARCHITECTURE, DEEP_ARCHITECTURE):
        network = NeuralPopulation(
            population,
            architecture,
            device=device,
        )

        inputs = torch.rand(
            population,
            architecture[0],
            device=device,
        ) * 2.0 - 1.0

        outputs = network.forward(inputs)

        print()
        print(f"Architecture: {architecture}")
        print(f"Population: {population:,}")
        print(f"Parameters per organism: {network.parameters_per_organism:,}")
        print(f"Total parameters: {network.parameter_count:,}")
        print(f"Parameter memory: {network.parameter_memory_bytes / 1e6:.2f} MB")
        print(f"Input shape: {tuple(inputs.shape)}")
        print(f"Output shape: {tuple(outputs.shape)}")
        print(f"Output range: [{outputs.min().item():.4f}, {outputs.max().item():.4f}]")
        print(f"Output device: {outputs.device}")
        print(f"Finite outputs: {torch.isfinite(outputs).all().item()}")


if __name__ == "__main__":
    main()
