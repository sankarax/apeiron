# Neural controllers

Each neural organism owns its parameters. Inference uses FP32 arithmetic. Parameters are inherited and mutated, not trained by gradient descent.

| Controller | Architecture | Parameters | Parameter bytes |
| --- | --- | ---: | ---: |
| Heuristic | Rule-based | 0 | 0 |
| Small | 6, 8, 3 | 83 | 332 |
| Deep | 16, 32, 16, 4 | 1140 | 4560 |
| Recurrent | 16 inputs, 16 memory, 4 outputs | 1652 | 6608 |

Recurrent memory adds 64 bytes. These counts exclude allocation overhead and organism metadata. Dense layers store weights as `[output][input]` and a bias per output. A layer with I inputs and O outputs has `O * (I + 1)` parameters. MLP layers apply tanh, including the output layer. Xavier initialization limits initial activation scale; biases start at zero.

## Inputs and actions

Sensors represent local food direction and distance, internal energy and other state. Deep inputs include neighborhood and species information. Direction features are expressed relative to the organism's heading. This replaced the original world-frame representation after the small network failed to survive the recorded experiments. See `sensors.rs` for exact feature order and normalization.

Controller outputs determine movement and reproductive behavior; the deep and recurrent controllers also provide aggression. The small controller supplies no aggression. Actions are bounded before use in the simulation.

## Recurrent controller

The recurrent network is a custom GRU-style implementation. With input x and previous memory h, it computes:

```text
z = sigmoid(Wz [x,h] + bz)
r = sigmoid(Wr [x,h] + br)
c = tanh(Wc [x,r*h] + bc)
h_next = (1-z)*h + z*c
y = tanh(Wo h_next + bo)
```

The gates and candidate each have 16 outputs and 32 inputs. The output layer has 4 outputs and 16 inputs. The total is `3 * 16 * 33 + 4 * 17 = 1652`. The update convention and reset placement are specific to this implementation; a stock PyTorch GRU is not a drop-in reference. Forward inference returns the new memory without mutating the network. The simulation commits it after inference.

## Mutation and inspection

Mutation samples parameters independently according to a configured probability and applies bounded random perturbations. Offspring receive independent copies of the network. Boxing neural variants keeps the inline enum small for heuristic organisms, at the cost of heap allocation and indirection.

The brain inspector displays inputs, weights, activations and recurrent gates and memory for a selected organism. It computes a preview from the current state; it is not a recording of the previous executed action and does not advance the simulation.

## GPU inference

Native wgpu shaders implement the same arithmetic. Organism-specific weights are packed into tiled buffers to improve access patterns. Stable slots allow unchanged weights to remain resident and recycled slots to be updated when organisms are born. Dispatches are batched and readback is synchronized before CPU ecology updates. The public WASM edition does not use these shaders.

The PyTorch reference in `experiments/` provides an independent numerical implementation and CUDA inference benchmarks. Numerical agreement is tolerance-based rather than bitwise across devices. See [performance](performance.md) for measured errors and throughput.
