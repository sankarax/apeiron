use std::mem::size_of;

use apeiron_engine::{ControllerKind, DeepNetwork, NeuralGenome, SmallNetwork};

fn main() {
    println!("Apeiron neural memory layout");
    println!("Pointer size: {} bytes", size_of::<usize>());
    println!("ControllerKind: {} bytes", size_of::<ControllerKind>());
    println!("NeuralGenome: {} bytes", size_of::<NeuralGenome>());
    println!("SmallNetwork: {} bytes", size_of::<SmallNetwork>());
    println!("DeepNetwork: {} bytes", size_of::<DeepNetwork>());
    println!(
        "Box<SmallNetwork>: {} bytes",
        size_of::<Box<SmallNetwork>>()
    );
    println!("Box<DeepNetwork>: {} bytes", size_of::<Box<DeepNetwork>>());

    println!();

    let population = 100_000;

    for (name, parameter_bytes) in [
        ("Heuristic", 0),
        ("Small", size_of::<SmallNetwork>()),
        ("Deep", size_of::<DeepNetwork>()),
    ] {
        let inline = population * size_of::<NeuralGenome>();
        let heap = population * parameter_bytes;
        let total = inline + heap;

        println!(
            "{name}: inline {:.2} MB, parameters {:.2} MB, total {:.2} MB",
            inline as f64 / 1e6,
            heap as f64 / 1e6,
            total as f64 / 1e6,
        );
    }
}
