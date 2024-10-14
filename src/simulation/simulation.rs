use crate::common::{Particle, MAX_PARTICLES};
use bytemuck::{Pod, Zeroable};
use encase::{ShaderSize, UniformBuffer};
use glam::Vec3;
use std::borrow::Cow;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, ComputePipeline, ComputePipelineDescriptor,
    Device, PipelineLayoutDescriptor, Queue, ShaderModuleDescriptor, ShaderSource, ShaderStages,
};

#[include_wgsl_oil::include_wgsl_oil("simulation.wgsl")]
mod shader {}
pub use shader::types::Uniforms;
unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}
impl Copy for Uniforms {}

pub struct Simulation {
    bind_group_layout: BindGroupLayout,
    simulate_compute_pipeline: ComputePipeline,
    uniform_buffer: Buffer,
    pub particle_buffer: Buffer,
}

impl Drop for Simulation {
    fn drop(&mut self) {}
}

impl Simulation {
    pub fn new(device: &Device) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(shader::SOURCE)),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Uniforms
                BindGroupLayoutEntry {
                    binding: shader::globals::uniforms::binding::BINDING,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Particles
                BindGroupLayoutEntry {
                    binding: shader::globals::particles::binding::BINDING,
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
                    binding: shader::globals::bounds::binding::BINDING,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Grid
                BindGroupLayoutEntry {
                    binding: shader::globals::grid::binding::BINDING,
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
                entry_point: shader::entry_points::simulate::NAME,
            });

        let uniform_buffer: Buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: Uniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let particle_buffer = device.create_buffer(&BufferDescriptor {
            size: Particle::SHADER_SIZE.get() * MAX_PARTICLES as u64,
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Simulation {
            bind_group_layout,
            simulate_compute_pipeline,
            uniform_buffer,
            particle_buffer,
        }
    }

    pub fn simulate(
        &self,
        device: &Device,
        queue: &Queue,
        bounds_buffer: &Buffer,
        grid_buffer: &Buffer,
        delta_time: f32,
        gravity: Vec3,
    ) {
        let uniforms = Uniforms {
            delta_time,
            gravity,
        };
        let mut encased_uniform_buffer = UniformBuffer::new(Vec::<u8>::new());
        encased_uniform_buffer.write(&uniforms).unwrap();
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            &encased_uniform_buffer.into_inner(),
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: shader::globals::uniforms::binding::BINDING,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: shader::globals::particles::binding::BINDING,
                    resource: self.particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: shader::globals::bounds::binding::BINDING,
                    resource: bounds_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: shader::globals::grid::binding::BINDING,
                    resource: grid_buffer.as_entire_binding(),
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
