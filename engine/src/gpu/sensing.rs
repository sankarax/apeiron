use super::GpuInference;
use crate::{ControllerKind, Food, Organism, SensorReading, SimulationConfig};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    width: f32,
    height: f32,
    cols: u32,
    rows: u32,
    count: u32,
    cells: u32,
    pad: [u32; 2],
}

fn cell(x: f32, y: f32, p: &Params) -> usize {
    let col = ((x.rem_euclid(p.width) / (p.width / p.cols as f32)).floor() as u32).min(p.cols - 1);
    let row =
        ((y.rem_euclid(p.height) / (p.height / p.rows as f32)).floor() as u32).min(p.rows - 1);
    (row * p.cols + col) as usize
}

fn append_grid(grid: &mut Vec<u32>, offset: usize, cells: Vec<Vec<u32>>) {
    let end = cells.len();
    for (i, entries) in cells.into_iter().enumerate() {
        grid[offset + i] = grid.len() as u32;
        grid.extend(entries);
    }
    grid[offset + end] = grid.len() as u32;
}

impl GpuInference {
    pub fn sense(
        &self,
        organisms: &[Organism],
        food: &[Food],
        config: &SimulationConfig,
    ) -> Result<Vec<SensorReading>, String> {
        self.sense_mode(organisms, food, config, false)
    }

    pub fn sense_mode(
        &self,
        organisms: &[Organism],
        food: &[Food],
        config: &SimulationConfig,
        all_cells: bool,
    ) -> Result<Vec<SensorReading>, String> {
        if organisms.is_empty() {
            return Ok(Vec::new());
        }
        let cols = ((config.width / 50.0).ceil() as u32).max(1);
        let rows = ((config.height / 50.0).ceil() as u32).max(1);
        let p = Params {
            width: config.width,
            height: config.height,
            cols,
            rows,
            count: organisms.len() as u32,
            cells: cols * rows,
            pad: [
                u32::from(all_cells),
                u32::from(organisms.iter().any(|o| {
                    matches!(
                        o.brain.kind(),
                        ControllerKind::Deep | ControllerKind::Recurrent
                    )
                })),
            ],
        };
        let mut fg = vec![Vec::new(); p.cells as usize];
        let mut og = vec![Vec::new(); p.cells as usize];
        let agents: Vec<[f32; 4]> = organisms
            .iter()
            .enumerate()
            .map(|(i, o)| {
                og[cell(o.x, o.y, &p)].push(i as u32);
                [
                    o.x.rem_euclid(p.width),
                    o.y.rem_euclid(p.height),
                    o.genome.sensor_radius,
                    if o.is_alive(config) { 1.0 } else { 0.0 },
                ]
            })
            .collect();
        let mut foods: Vec<[f32; 2]> = food
            .iter()
            .enumerate()
            .map(|(i, f)| {
                fg[cell(f.x, f.y, &p)].push(i as u32);
                [f.x, f.y]
            })
            .collect();
        if foods.is_empty() {
            foods.push([0.0; 2]);
        }
        let mut grid = vec![0u32; 2 * (p.cells as usize + 1)];
        append_grid(&mut grid, 0, fg);
        append_grid(&mut grid, p.cells as usize + 1, og);
        let buffer = |bytes: &[u8], usage| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytes,
                    usage,
                })
        };
        let params = buffer(bytemuck::bytes_of(&p), wgpu::BufferUsages::UNIFORM);
        let agents = buffer(bytemuck::cast_slice(&agents), wgpu::BufferUsages::STORAGE);
        let foods = buffer(bytemuck::cast_slice(&foods), wgpu::BufferUsages::STORAGE);
        let grid = buffer(bytemuck::cast_slice(&grid), wgpu::BufferUsages::STORAGE);
        let size = organisms.len() as u64 * 36;
        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let entries: Vec<_> = [&params, &agents, &foods, &grid, &output]
            .iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.sensing.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.sensing);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(p.count.div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, size);
        self.queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("{e:?}"))?;
        rx.recv()
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let mut readings: Vec<SensorReading> = {
            let bytes = slice.get_mapped_range();
            organisms
                .iter()
                .zip(bytemuck::cast_slice::<u8, [f32; 9]>(&bytes))
                .map(|(o, v)| {
                    let mut r = SensorReading::from_organism(o, config);
                    r.food_distance = v[0];
                    r.food_direction_x = v[1];
                    r.food_direction_y = v[2];
                    r.food_density = v[3];
                    r.food_detected = v[4] > 0.5;
                    if matches!(
                        o.brain.kind(),
                        ControllerKind::Deep | ControllerKind::Recurrent
                    ) {
                        r.organism_distance = v[5];
                        r.organism_direction_x = v[6];
                        r.organism_direction_y = v[7];
                        r.organism_density = v[8];
                    }
                    r
                })
                .collect()
        };
        readback.unmap();
        if config.predation {
            let mut grid = crate::SpatialGrid::new(config.width, config.height, 50.0);
            grid.rebuild_organisms(organisms);
            for (reading, organism) in readings.iter_mut().zip(organisms) {
                crate::sensory_pipeline::apply_species_sensors(
                    reading,
                    organism,
                    organisms,
                    Some(&grid),
                    config,
                );
            }
        }
        Ok(readings)
    }
}
