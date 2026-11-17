
import pytest
import torch
import torch.nn.functional as F

from neural_inference import (
    BatchedDense,
    NeuralPopulation,
    SMALL_ARCHITECTURE,
    DEEP_ARCHITECTURE,
)


@pytest.mark.parametrize(
    "architecture, expected",
    [
        (SMALL_ARCHITECTURE, 83),
        (DEEP_ARCHITECTURE, 1140),
    ],
)
def test_parameter_count(architecture, expected):
    network = NeuralPopulation(100, architecture)

    assert network.parameters_per_organism == expected
    assert network.parameter_count == 100 * expected
    assert network.parameter_memory_bytes == 100 * expected * 4


def test_dense_forward():
    layer = BatchedDense(
        weights=torch.tensor([
            [[1.0, 2.0], [-1.0, 3.0]],
        ]),
        biases=torch.tensor([[0.5, -0.5]]),
    )

    inputs = torch.tensor([[2.0, 3.0]])
    expected = torch.tensor([[8.5, 6.5]])

    torch.testing.assert_close(layer.forward(inputs), expected)


def test_independent_weights():
    layer = BatchedDense(
        weights=torch.tensor([
            [[1.0, 2.0]],
            [[3.0, 4.0]],
        ]),
        biases=torch.zeros(2, 1),
    )

    inputs = torch.tensor([
        [1.0, 1.0],
        [1.0, 1.0],
    ])

    expected = torch.tensor([[3.0], [7.0]])

    torch.testing.assert_close(layer.forward(inputs), expected)


def test_deterministic_initialization():
    a = NeuralPopulation(100, seed=42)
    b = NeuralPopulation(100, seed=42)

    for layer_a, layer_b in zip(a.layers, b.layers):
        assert torch.equal(layer_a.weights, layer_b.weights)
        assert torch.equal(layer_a.biases, layer_b.biases)


def test_different_seeds():
    a = NeuralPopulation(100, seed=42)
    b = NeuralPopulation(100, seed=43)

    assert not torch.equal(
        a.layers[0].weights,
        b.layers[0].weights,
    )


@pytest.mark.parametrize(
    "architecture",
    [SMALL_ARCHITECTURE, DEEP_ARCHITECTURE],
)
def test_output_shape_and_bounds(architecture):
    network = NeuralPopulation(100, architecture)
    inputs = torch.randn(100, architecture[0])

    outputs = network.forward(inputs)

    assert outputs.shape == (100, architecture[-1])
    assert torch.isfinite(outputs).all()
    assert (outputs.abs() <= 1.0).all()


def test_zero_inputs():
    network = NeuralPopulation(100)
    inputs = torch.zeros(100, DEEP_ARCHITECTURE[0])

    outputs = network.forward(inputs)

    assert torch.equal(outputs, torch.zeros_like(outputs))


def test_invalid_input_shape():
    network = NeuralPopulation(100)

    with pytest.raises(ValueError):
        network.forward(torch.zeros(50, 16))


@pytest.mark.parametrize(
    "architecture",
    [SMALL_ARCHITECTURE, DEEP_ARCHITECTURE],
)
def test_matches_individual_inference(architecture):
    population = 32
    network = NeuralPopulation(population, architecture)
    inputs = torch.randn(population, architecture[0])

    batched_outputs = network.forward(inputs)
    individual_outputs = []

    for i in range(population):
        output = inputs[i]

        for layer in network.layers:
            output = torch.tanh(
                F.linear(
                    output,
                    layer.weights[i],
                    layer.biases[i],
                )
            )

        individual_outputs.append(output)

    expected = torch.stack(individual_outputs)

    torch.testing.assert_close(
        batched_outputs,
        expected,
        rtol=1e-5,
        atol=1e-6,
    )


@pytest.mark.skipif(
    not torch.cuda.is_available(),
    reason="CUDA unavailable",
)
@pytest.mark.parametrize(
    "architecture",
    [SMALL_ARCHITECTURE, DEEP_ARCHITECTURE],
)
def test_cpu_gpu_equivalence(architecture):
    population = 1000

    cpu = NeuralPopulation(
        population,
        architecture,
        device="cpu",
    )

    gpu = NeuralPopulation(
        population,
        architecture,
        device="cuda",
    )

    for cpu_layer, gpu_layer in zip(cpu.layers, gpu.layers):
        gpu_layer.weights.copy_(cpu_layer.weights)
        gpu_layer.biases.copy_(cpu_layer.biases)

    inputs = torch.randn(population, architecture[0])

    cpu_outputs = cpu.forward(inputs)
    gpu_outputs = gpu.forward(inputs.cuda()).cpu()

    torch.testing.assert_close(
        cpu_outputs,
        gpu_outputs,
        rtol=1e-5,
        atol=1e-6,
    )
