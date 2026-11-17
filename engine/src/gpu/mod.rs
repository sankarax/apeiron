use crate::{ControllerKind, DenseLayer, NeuralGenome, SensorReading};
use std::sync::Mutex;

pub struct GpuInference {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: [wgpu::ComputePipeline; 3],
    sensing: wgpu::ComputePipeline,
    neural_buffers: Mutex<Vec<NeuralBuffers>>,
    pub adapter: wgpu::AdapterInfo,
}

struct NeuralBuffers {
    input: wgpu::Buffer,
    weights: wgpu::Buffer,
    ids: Vec<Option<u64>>,
    host_weights: Vec<f32>,
    output: wgpu::Buffer,
    readback: wgpu::Buffer,
    bind: wgpu::BindGroup,
    kind: ControllerKind,
}

pub type GpuOutput = [f32; 20];

fn pack_layer<const I: usize, const O: usize>(data: &mut Vec<f32>, layer: &DenseLayer<I, O>) {
    for row in &layer.weights {
        data.extend_from_slice(row);
    }
    data.extend_from_slice(&layer.biases);
}

impl GpuInference {
    pub fn new() -> Result<Self, String> {
        pollster::block_on(Self::initialize())
    }

    async fn initialize() -> Result<Self, String> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| e.to_string())?;
        let pipelines = ["small", "deep", "recurrent"].map(|entry| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("apeiron neural inference"),
                source: wgpu::ShaderSource::Wgsl(include_str!("neural.wgsl").into()),
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: None,
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("spatial sensing"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sensing.wgsl").into()),
        });
        let sensing = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("spatial sensing"),
            layout: None,
            module: &module,
            entry_point: Some("sense"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Self {
            device,
            queue,
            pipelines,
            sensing,
            neural_buffers: Mutex::new(Vec::new()),
            adapter: info,
        })
    }

    pub fn invalidate_weights(&self) {
        if let Ok(mut cache) = self.neural_buffers.lock() {
            for b in cache.iter_mut() {
                b.ids.fill(None);
            }
        }
    }

    pub fn infer(
        &self,
        kind: ControllerKind,
        brains: &[&NeuralGenome],
        sensors: &[SensorReading],
    ) -> Result<Vec<GpuOutput>, String> {
        self.infer_inner(kind, brains, sensors, None)
    }

    pub(crate) fn infer_resident(
        &self,
        kind: ControllerKind,
        brains: &[&NeuralGenome],
        sensors: &[SensorReading],
        ids: &[u64],
    ) -> Result<Vec<GpuOutput>, String> {
        if ids.len() != brains.len() {
            return Err("id count differs".into());
        }
        self.infer_inner(kind, brains, sensors, Some(ids))
    }

    fn infer_inner(
        &self,
        kind: ControllerKind,
        brains: &[&NeuralGenome],
        sensors: &[SensorReading],
        ids: Option<&[u64]>,
    ) -> Result<Vec<GpuOutput>, String> {
        if brains.len() != sensors.len() {
            return Err("brain and sensor counts differ".into());
        }
        let (pipeline, inputs, params) = match kind {
            ControllerKind::Small => (&self.pipelines[0], 6, 83),
            ControllerKind::Deep => (&self.pipelines[1], 16, 1140),
            ControllerKind::Recurrent => (&self.pipelines[2], 32, 1652),
            ControllerKind::Heuristic => return Err("heuristic has no neural inference".into()),
        };
        const CAPACITY: usize = 8192;
        const TILE: usize = 16;
        if brains.iter().any(|brain| brain.kind() != kind) {
            return Err("mixed controller batch".into());
        }
        let mut cache = self.neural_buffers.lock().map_err(|e| e.to_string())?;
        if cache.first().is_some_and(|b| b.kind != kind) {
            cache.clear();
        }
        if brains.is_empty() {
            cache.clear();
            return Ok(Vec::new());
        }
        let resident = ids.is_some();
        let keys: Vec<u64> = ids.map_or_else(|| (0..brains.len() as u64).collect(), |v| v.to_vec());
        let live: std::collections::HashSet<_> = keys.iter().copied().collect();
        if live.len() != keys.len() {
            return Err("duplicate organism ids".into());
        }
        let mut slots = std::collections::HashMap::new();
        let mut free = Vec::new();
        for (chunk, buffers) in cache.iter_mut().enumerate() {
            for (slot, id) in buffers.ids.iter_mut().enumerate() {
                if let Some(key) = id.filter(|key| resident && live.contains(key)) {
                    slots.insert(key, (chunk, slot));
                } else {
                    *id = None;
                    free.push((chunk, slot));
                }
            }
        }
        free.reverse();
        let mut assignments = Vec::with_capacity(brains.len());
        let mut dirty: Vec<Vec<bool>> =
            cache.iter().map(|_| vec![false; CAPACITY / TILE]).collect();
        let mut packed = Vec::with_capacity(params);
        for (index, &id) in keys.iter().enumerate() {
            if let Some(&slot) = slots.get(&id) {
                assignments.push(slot);
                continue;
            }
            if free.is_empty() {
                let make = |size, usage| {
                    self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: None,
                        size,
                        usage,
                        mapped_at_creation: false,
                    })
                };
                let size = (CAPACITY * std::mem::size_of::<GpuOutput>()) as u64;
                let input = make(
                    (CAPACITY * inputs * 4) as u64,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                );
                let weights = make(
                    (CAPACITY * params * 4) as u64,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                );
                let output = make(
                    size,
                    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                );
                let readback = make(
                    size,
                    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                );
                let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: input.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: output.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: weights.as_entire_binding(),
                        },
                    ],
                });
                let chunk = cache.len();
                cache.push(NeuralBuffers {
                    input,
                    weights,
                    output,
                    readback,
                    bind,
                    kind,
                    ids: vec![None; CAPACITY],
                    host_weights: vec![0.0; CAPACITY * params],
                });
                dirty.push(vec![false; CAPACITY / TILE]);
                free.extend((0..CAPACITY).rev().map(|slot| (chunk, slot)));
            }
            let (chunk, slot) = free.pop().unwrap();
            let buffers = &mut cache[chunk];
            buffers.ids[slot] = Some(id);
            packed.clear();
            match (kind, brains[index]) {
                (ControllerKind::Small, NeuralGenome::Small(n)) => {
                    pack_layer(&mut packed, &n.hidden);
                    pack_layer(&mut packed, &n.output);
                }
                (ControllerKind::Deep, NeuralGenome::Deep(n)) => {
                    pack_layer(&mut packed, &n.hidden_1);
                    pack_layer(&mut packed, &n.hidden_2);
                    pack_layer(&mut packed, &n.output);
                }
                (ControllerKind::Recurrent, NeuralGenome::Recurrent(n)) => {
                    pack_layer(&mut packed, &n.update);
                    pack_layer(&mut packed, &n.reset);
                    pack_layer(&mut packed, &n.candidate);
                    pack_layer(&mut packed, &n.output);
                }
                _ => return Err("mixed controller batch".into()),
            }
            let tile = slot / TILE;
            for (j, &weight) in packed.iter().enumerate() {
                buffers.host_weights[tile * params * TILE + j * TILE + slot % TILE] = weight;
            }
            dirty[chunk][tile] = true;
            assignments.push((chunk, slot));
        }
        let mut counts = vec![0; cache.len()];
        for &(chunk, slot) in &assignments {
            counts[chunk] = counts[chunk].max(slot + 1);
        }
        let mut data: Vec<Vec<f32>> = counts.iter().map(|&n| vec![0.0; n * inputs]).collect();
        for ((brain, sensor), &(chunk, slot)) in brains.iter().zip(sensors).zip(&assignments) {
            let row = &mut data[chunk][slot * inputs..(slot + 1) * inputs];
            match (kind, *brain) {
                (ControllerKind::Small, NeuralGenome::Small(_)) => {
                    row.copy_from_slice(&sensor.small_inputs())
                }
                (ControllerKind::Deep, NeuralGenome::Deep(_)) => {
                    row.copy_from_slice(&sensor.deep_inputs())
                }
                (ControllerKind::Recurrent, NeuralGenome::Recurrent(n)) => {
                    row[..16].copy_from_slice(&sensor.deep_inputs());
                    row[16..].copy_from_slice(&n.memory);
                }
                _ => return Err("mixed controller batch".into()),
            }
        }
        let mut encoder = self.device.create_command_encoder(&Default::default());
        for (chunk, buffers) in cache.iter().enumerate() {
            let count = counts[chunk];
            if count == 0 {
                continue;
            }
            let mut tile = 0;
            while tile < dirty[chunk].len() {
                if !dirty[chunk][tile] {
                    tile += 1;
                    continue;
                }
                let start = tile;
                while tile < dirty[chunk].len() && dirty[chunk][tile] {
                    tile += 1;
                }
                let range = start * params * TILE..tile * params * TILE;
                self.queue.write_buffer(
                    &buffers.weights,
                    (range.start * 4) as u64,
                    bytemuck::cast_slice(&buffers.host_weights[range]),
                );
            }
            self.queue
                .write_buffer(&buffers.input, 0, bytemuck::cast_slice(&data[chunk]));
            {
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &buffers.bind, &[]);
                pass.dispatch_workgroups((count as u32).div_ceil(64), 1, 1);
            }
            encoder.copy_buffer_to_buffer(
                &buffers.output,
                0,
                &buffers.readback,
                0,
                (count * std::mem::size_of::<GpuOutput>()) as u64,
            );
        }
        self.queue.submit([encoder.finish()]);
        let receivers: Vec<_> = cache
            .iter()
            .zip(&counts)
            .map(|(buffers, &count)| {
                if count == 0 {
                    return None;
                }
                let (tx, rx) = std::sync::mpsc::channel();
                buffers
                    .readback
                    .slice(..(count * std::mem::size_of::<GpuOutput>()) as u64)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        let _ = tx.send(result);
                    });
                Some(rx)
            })
            .collect();
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("{e:?}"))?;
        let mut rows = Vec::with_capacity(cache.len());
        for ((buffers, rx), &count) in cache.iter().zip(receivers).zip(&counts) {
            if let Some(rx) = rx {
                rx.recv()
                    .map_err(|e| e.to_string())?
                    .map_err(|e| e.to_string())?;
                {
                    let bytes = buffers
                        .readback
                        .slice(..(count * std::mem::size_of::<GpuOutput>()) as u64)
                        .get_mapped_range();
                    rows.push(bytemuck::cast_slice::<u8, GpuOutput>(&bytes).to_vec());
                }
                buffers.readback.unmap();
            } else {
                rows.push(Vec::new());
            }
        }
        let outputs = assignments
            .into_iter()
            .map(|(chunk, slot)| rows[chunk][slot])
            .collect();
        if !resident {
            for b in cache.iter_mut() {
                b.ids.fill(None);
            }
        }

        Ok(outputs)
    }
}
mod sensing;
