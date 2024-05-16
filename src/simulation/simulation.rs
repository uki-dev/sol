use std::{borrow::Cow, mem::size_of};

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, ComputePipeline, ComputePipelineDescriptor,
    Device, PipelineLayoutDescriptor, Queue, ShaderModuleDescriptor, ShaderSource, ShaderStages,
};

use encase::ShaderSize;

use crate::common::{Particle, MAX_PARTICLES};

#[include_wgsl_oil::include_wgsl_oil("simulation.wgsl")]
mod simulation_shader {}

pub struct Simulation {
    pub particle_buffer: Buffer,

    bind_group_layout: BindGroupLayout,
    // populate_compute_pipeline: ComputePipeline,
    simulate_compute_pipeline: ComputePipeline,
}

impl Drop for Simulation {
    fn drop(&mut self) {}
}

impl Simulation {
    pub fn new(device: &Device) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(simulation_shader::SOURCE)),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Particles
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Bounds
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let simulate_compute_pipeline =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "simulate",
            });

        let particle_buffer = device.create_buffer(&BufferDescriptor {
            size: Particle::SHADER_SIZE.get() * MAX_PARTICLES as u64,
            label: Some("Simulation::storage_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Simulation {
            particle_buffer,
            bind_group_layout,
            simulate_compute_pipeline,
        }
    }

    fn dispatch(&self, device: &Device, queue: &Queue, pipeline: &ComputePipeline) {}

    pub fn simulate(&mut self, device: &Device, queue: &Queue, bounds_buffer: &Buffer) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: bounds_buffer.as_entire_binding(),
                },
            ],
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.simulate_compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(MAX_PARTICLES, 1, 1);
        }
        queue.submit(Some(encoder.finish()));
    }
}
