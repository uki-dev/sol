use futures::channel::oneshot;
use std::{borrow::Cow, mem::size_of};
// use ultraviolet::Vec3;
use wgpu::{
    BindGroup, Buffer, BufferAsyncError, BufferDescriptor, BufferUsages, ComputePipeline, Device,
    Queue,
};

#[path = "cell.rs"]
mod cell;
use cell::Cell;

pub struct SimulationDescriptor {
    /// number of cells within x axis of the discrete simulation grid
    pub width: u32,
    /// number of cells within y axis of the discrete simulation grid
    pub height: u32,
    /// number of cells within z axis of the discrete simulation grid
    pub depth: u32,
}

pub struct Simulation {
    /// number of cells within x axis of the discrete simulation grid
    pub width: u32,
    /// number of cells within y axis of the discrete simulation grid
    pub height: u32,
    /// number of cells within z axis of the discrete simulation grid
    pub depth: u32,

    staging_buffer: Buffer,
    pub storage_buffer: Buffer,
    compute_pipeline: ComputePipeline,
    bind_group: BindGroup,
}

impl Drop for Simulation {
    fn drop(&mut self) {
        self.staging_buffer.destroy();
        self.storage_buffer.destroy();
    }
}

impl Simulation {
    pub fn new(descriptor: &SimulationDescriptor, device: &Device) -> Self {
        let (width, height, depth) = (descriptor.width, descriptor.height, descriptor.depth);
        let size = (size_of::<Cell>() as u32 * width * height * depth) as u64;

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("simulation.wgsl"))),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader_module,
            entry_point: "main",
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size,
            label: None,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer(&BufferDescriptor {
            size,
            label: Some("Simulation::storage_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
        });

        Simulation {
            width: descriptor.width,
            height: descriptor.height,
            depth: descriptor.depth,
            staging_buffer,
            storage_buffer,
            compute_pipeline,
            bind_group,
        }
    }

    pub fn dispatch(&self, device: &Device, queue: &Queue) {
        let (staging_buffer, storage_buffer, compute_pipeline, bind_group) = (
            &self.staging_buffer,
            &self.storage_buffer,
            &self.compute_pipeline,
            &self.bind_group,
        );

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(compute_pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            compute_pass.dispatch_workgroups(self.width, self.height, self.depth);
        }
        let size = (size_of::<Cell>() as u32 * self.width * self.height * self.depth) as u64;
        encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);
        queue.submit(Some(encoder.finish()));
    }

    pub async fn receive(&self, device: &Device) -> Result<(), BufferAsyncError> {
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
