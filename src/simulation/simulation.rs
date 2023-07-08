use futures::channel::oneshot;
use std::{borrow::Cow, mem::size_of};
// use ultraviolet::Vec3;
use wgpu::{
    BindGroup, Buffer, BufferAsyncError, BufferDescriptor, BufferUsages, ComputePipeline, Device,
    PipelineLayoutDescriptor, Queue,
};

#[path = "cell.rs"]
mod cell;
use cell::Cell;

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Uniforms {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub _padding_0: u32,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

pub struct Simulation {
    /// number of cells within x axis of the discrete simulation grid
    width: u32,
    /// number of cells within y axis of the discrete simulation grid
    height: u32,
    /// number of cells within z axis of the discrete simulation grid
    depth: u32,

    /// denotes whether `uniform_buffer` is dirty and needs updating
    dirty: bool,

    uniform_buffer: Buffer,
    pub storage_buffer: Buffer,
    staging_buffer: Buffer,

    bind_group: BindGroup,

    populate_compute_pipeline: ComputePipeline,
    simulate_compute_pipeline: ComputePipeline,
}

impl Drop for Simulation {
    fn drop(&mut self) {
        self.uniform_buffer.destroy();
        self.staging_buffer.destroy();
        self.storage_buffer.destroy();
    }
}

impl Simulation {
    pub fn width(&self) -> u32 {
        return self.width;
    }

    pub fn height(&self) -> u32 {
        return self.height;
    }

    pub fn depth(&self) -> u32 {
        return self.depth;
    }

    pub fn set_dimensions(&mut self, width: u32, height: u32, depth: u32, device: &Device) {
        if self.width != width || self.height != height || self.depth != depth {
            self.dirty = true;
        }
        self.width = width;
        self.height = height;
        self.depth = depth;

        self.storage_buffer.destroy();
        self.staging_buffer.destroy();

        let size = (size_of::<Cell>() as u32 * width * height * depth) as u64;

        let storage_buffer = device.create_buffer(&BufferDescriptor {
            size,
            label: Some("Simulation::storage_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            label: None,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.storage_buffer = storage_buffer;
        self.staging_buffer = staging_buffer;

        // TODO: copy contents of old buffer
    }

    pub fn new(width: u32, height: u32, depth: u32, device: &Device) -> Self {
        let size = (size_of::<Cell>() as u32 * width * height * depth) as u64;

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("simulation.wgsl"))),
        });

        let uniform_buffer: wgpu::Buffer = device.create_buffer(&BufferDescriptor {
            label: Some("uniforms"),
            size: size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer(&BufferDescriptor {
            size,
            label: Some("Simulation::storage_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            label: None,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: storage_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let populate_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "populate",
            });

        let simulate_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "simulate",
            });

        Simulation {
            width,
            height,
            depth,
            uniform_buffer,
            storage_buffer,
            staging_buffer,
            bind_group,
            populate_compute_pipeline,
            simulate_compute_pipeline,
            dirty: true,
        }
    }

    fn update(&mut self, queue: &Queue) {
        if self.dirty {
            let uniforms = Uniforms {
                width: self.width,
                height: self.height,
                depth: self.depth,
                ..Default::default()
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
            self.dirty = false;
        }
    }

    fn dispatch(&self, device: &Device, queue: &Queue, pipeline: &ComputePipeline) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(self.width, self.height, self.depth);
        }
        queue.submit(Some(encoder.finish()));
    }

    pub fn populate(&mut self, device: &Device, queue: &Queue) {
        self.update(queue);
        self.dispatch(device, queue, &self.populate_compute_pipeline);
    }

    pub fn simulate(&mut self, device: &Device, queue: &Queue) {
        self.update(queue);
        self.dispatch(device, queue, &self.simulate_compute_pipeline);
    }

    pub async fn receive(&self, device: &Device, queue: &Queue) -> Result<(), BufferAsyncError> {
        let size = (size_of::<Cell>() as u32 * self.width * self.height * self.depth) as u64;
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&self.storage_buffer, 0, &self.staging_buffer, 0, size);
        queue.submit(Some(encoder.finish()));

        let staging_buffer = &self.staging_buffer;
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::Wait);

        match &receiver.await {
            Ok(Err(error)) => return Err(error.clone()),
            // TODO: should we throw error if buffer mapping is cancelled ?
            _ => {}
        };

        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice::<u8, Cell>(&data).to_vec();

        for (i, cell) in result.iter().enumerate() {
            let i_u32: u32 = i as u32;
            let z = i_u32 / (self.width * self.height);
            let y = (i_u32 / self.width) % self.height;
            let x = i_u32 % self.width;
            println!("x: {}, y: {}, z: {}, cell: {:?}", x, y, z, cell);
        }

        Ok(())
    }
}
