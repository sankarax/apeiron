use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use apeiron_engine::{DeepNetwork, DenseLayer, RecurrentNetwork, SmallNetwork};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;

const POPULATION: usize = 1000;
const SEED: u64 = 42;

#[derive(Serialize)]
struct LayerData {
    weights: Vec<Vec<Vec<f32>>>,
    biases: Vec<Vec<f32>>,
}

impl LayerData {
    fn new() -> Self {
        Self {
            weights: Vec::with_capacity(POPULATION),
            biases: Vec::with_capacity(POPULATION),
        }
    }

    fn push<const IN: usize, const OUT: usize>(&mut self, layer: &DenseLayer<IN, OUT>) {
        self.weights
            .push(layer.weights.iter().map(|row| row.to_vec()).collect());

        self.biases.push(layer.biases.to_vec());
    }
}

#[derive(Serialize)]
struct NeuralDataset {
    seed: u64,
    population: usize,
    architecture: Vec<usize>,
    layers: Vec<LayerData>,
    inputs: Vec<Vec<f32>>,
    outputs: Vec<Vec<f32>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    memory: Vec<Vec<f32>>,
}

impl NeuralDataset {
    fn new(architecture: &[usize]) -> Self {
        Self {
            seed: SEED,
            population: POPULATION,
            architecture: architecture.to_vec(),
            layers: (0..architecture.len() - 1)
                .map(|_| LayerData::new())
                .collect(),
            inputs: Vec::with_capacity(POPULATION),
            outputs: Vec::with_capacity(POPULATION),
            memory: Vec::new(),
        }
    }

    fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, self)?;

        Ok(())
    }
}

fn random_inputs<const N: usize>(rng: &mut impl Rng) -> [f32; N] {
    std::array::from_fn(|_| rng.random_range(-1.0..1.0))
}

fn randomize_biases<const IN: usize, const OUT: usize>(
    layer: &mut DenseLayer<IN, OUT>,
    rng: &mut impl Rng,
) {
    for bias in &mut layer.biases {
        *bias = rng.random_range(-0.5..0.5);
    }
}

fn export_small() -> NeuralDataset {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut dataset = NeuralDataset::new(&[6, 8, 3]);

    for _ in 0..POPULATION {
        let mut network = SmallNetwork::random(&mut rng);

        randomize_biases(&mut network.hidden, &mut rng);
        randomize_biases(&mut network.output, &mut rng);

        let inputs = random_inputs::<6>(&mut rng);
        let outputs = network.forward(&inputs);

        dataset.layers[0].push(&network.hidden);
        dataset.layers[1].push(&network.output);

        dataset.inputs.push(inputs.to_vec());
        dataset.outputs.push(outputs.to_vec());
    }

    dataset
}

fn export_deep() -> NeuralDataset {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut dataset = NeuralDataset::new(&[16, 32, 16, 4]);

    for _ in 0..POPULATION {
        let mut network = DeepNetwork::random(&mut rng);

        randomize_biases(&mut network.hidden_1, &mut rng);
        randomize_biases(&mut network.hidden_2, &mut rng);
        randomize_biases(&mut network.output, &mut rng);

        let inputs = random_inputs::<16>(&mut rng);
        let outputs = network.forward(&inputs);

        dataset.layers[0].push(&network.hidden_1);
        dataset.layers[1].push(&network.hidden_2);
        dataset.layers[2].push(&network.output);

        dataset.inputs.push(inputs.to_vec());
        dataset.outputs.push(outputs.to_vec());
    }

    dataset
}

fn export_recurrent() -> NeuralDataset {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut dataset = NeuralDataset::new(&[16, 16, 4]);
    dataset.layers = (0..4).map(|_| LayerData::new()).collect();
    for _ in 0..POPULATION {
        let mut net = RecurrentNetwork::random(&mut rng);
        randomize_biases(&mut net.update, &mut rng);
        randomize_biases(&mut net.reset, &mut rng);
        randomize_biases(&mut net.candidate, &mut rng);
        randomize_biases(&mut net.output, &mut rng);
        net.memory = random_inputs(&mut rng);
        let inputs = random_inputs(&mut rng);
        let (output, memory) = net.forward(&inputs);
        dataset.layers[0].push(&net.update);
        dataset.layers[1].push(&net.reset);
        dataset.layers[2].push(&net.candidate);
        dataset.layers[3].push(&net.output);
        dataset.inputs.push(inputs.to_vec());
        dataset.memory.push(net.memory.to_vec());
        dataset
            .outputs
            .push(output.into_iter().chain(memory).collect());
    }
    dataset
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../experiments/data");

    std::fs::create_dir_all(&directory)?;

    let small = export_small();
    let deep = export_deep();

    let small_path = directory.join("small_reference.json");
    let deep_path = directory.join("deep_reference.json");

    small.save(&small_path)?;
    deep.save(&deep_path)?;

    println!("Apeiron neural reference datasets");
    println!("Seed: {SEED}");
    println!("Population: {POPULATION}");

    println!(
        "Small MLP: {} parameters per organism",
        SmallNetwork::parameter_count()
    );

    println!(
        "Deep MLP: {} parameters per organism",
        DeepNetwork::parameter_count()
    );

    let recurrent_path = directory.join("recurrent_reference.json");
    export_recurrent().save(&recurrent_path)?;
    for path in [small_path, deep_path, recurrent_path] {
        let size = std::fs::metadata(&path)?.len();

        println!("{} ({:.2} MB)", path.display(), size as f64 / 1e6);
    }

    Ok(())
}
