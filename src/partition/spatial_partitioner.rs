use std::{borrow::Cow, mem::size_of};

use wgpu::{
    BindGroup, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, ComputePipeline, Device,
    PipelineLayoutDescriptor, Queue,
};

#[path = "../data.rs"]
mod data;
use data::Bounds;

pub struct SpatialPartioner {
    max_particles: usize,

    pub bounds_buffer: Buffer,

    compute_bounds_pipeline: ComputePipeline,

    bind_group: BindGroup,
}

impl Drop for SpatialPartioner {
    fn drop(&mut self) {
        self.bounds_buffer.destroy();
    }
}

impl SpatialPartioner {
    pub fn new(device: &Device, particle_buffer: &Buffer, max_particles: usize) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "spatial_partitioner.wgsl"
            ))),
        });

        let bounds_buffer = device.create_buffer(&BufferDescriptor {
            size: size_of::<Bounds>() as u64,
            label: Some("SpatialPartioner::bounds_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Particles
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Bounds
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
                // Particles
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                // Bounds
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: bounds_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_bounds_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "compute_bounds",
            });

        SpatialPartioner {
            max_particles,
            bounds_buffer,
            compute_bounds_pipeline,
            bind_group,
        }
    }

    pub fn compute_bounds_with_encoder(&self, queue: &Queue, command_encoder: &mut CommandEncoder) {
        queue.write_buffer(
            &self.bounds_buffer,
            0,
            bytemuck::bytes_of(&Bounds {
                min: [0, 0, 0],
                max: [0, 0, 0],
            }),
        );
        let mut compute_pass =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(&self.compute_bounds_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups((self.max_particles as f32 / 256.0).ceil() as u32, 1, 1);
    }

    pub fn compute_bounds(&self, device: &Device, queue: &Queue) {
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.compute_bounds_with_encoder(queue, &mut command_encoder);
        queue.submit(Some(command_encoder.finish()));
    }
}
