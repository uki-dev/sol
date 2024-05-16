use crate::common::{Bounds, MAX_PARTICLES};
use bytemuck::Zeroable;
use encase::{ShaderSize, StorageBuffer};
use std::borrow::Cow;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, Queue, ShaderModuleDescriptor,
    ShaderSource, ShaderStages,
};

#[include_wgsl_oil::include_wgsl_oil("bounds.wgsl")]
mod bounds_shader {}

pub struct BoundsPartition {
    bind_group_layout: BindGroupLayout,
    calculate_bounds_pipeline: ComputePipeline,
    pub bounds_buffer: Buffer,
}

impl Drop for BoundsPartition {
    fn drop(&mut self) {
        self.bounds_buffer.destroy();
    }
}

impl BoundsPartition {
    pub fn new(device: &Device) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(bounds_shader::SOURCE)),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: bounds_shader::globals::particles::binding::BINDING,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: bounds_shader::globals::bounds::binding::BINDING,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
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

        let calculate_bounds_pipeline =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: bounds_shader::entry_points::calculate_bounds::NAME,
            });

        let bounds_buffer = device.create_buffer(&BufferDescriptor {
            size: Bounds::SHADER_SIZE.get(),
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        BoundsPartition {
            bind_group_layout,
            calculate_bounds_pipeline,
            bounds_buffer,
        }
    }

    pub fn calculate_bounds_with_encoder(
        &self,
        device: &Device,
        queue: &Queue,
        command_encoder: &mut CommandEncoder,
        particle_buffer: &Buffer,
    ) {
        let mut encased_bounds_buffer = StorageBuffer::new(Vec::<u8>::new());
        encased_bounds_buffer.write(&Bounds::zeroed()).unwrap();
        queue.write_buffer(&self.bounds_buffer, 0, &encased_bounds_buffer.into_inner());

        // TODO: We create a new bind group for every compute just as a way to dependency inject
        // the particle buffer outside of the constructor,(which can get complicated due to bi-directional dependencies)
        //
        // See:
        // let simulation = Simulation:new(bounds.bounds_buffer, ...);
        // let bounds = Bounds:new(simulation.particle_buffer, ...);
        //
        // We could do something like:
        // let simulation_resources = create_simulation_resources();
        // let bounds_resources = create_bounds_resources();
        // let simulation = Simulation:new(bounds_resources.bounds_buffer, ...);
        // let bounds = Bounds:new(simulation_resources.particle_buffer, ...);
        //
        // Otherwise we just keep the following:
        // let simulation = Simulation:new(...);
        // let bounds = Bounds:new(...);
        // bounds.calculate(&simulation.particle_buffer)
        // simulation.simulate(&bounds.bounds_buffer)

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: bounds_shader::globals::particles::binding::BINDING,
                    resource: particle_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: bounds_shader::globals::bounds::binding::BINDING,
                    resource: self.bounds_buffer.as_entire_binding(),
                },
            ],
        });
        let mut compute_pass =
            command_encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(&self.calculate_bounds_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        let workgroup_size = bounds_shader::entry_points::calculate_bounds::WORKGROUP_SIZE;
        compute_pass.dispatch_workgroups(
            (MAX_PARTICLES as f32 / workgroup_size[0] as f32).ceil() as u32,
            workgroup_size[1],
            workgroup_size[2],
        );
    }

    pub fn calculate_bounds(&self, device: &Device, queue: &Queue, particle_buffer: &Buffer) {
        let mut command_encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        self.calculate_bounds_with_encoder(device, queue, &mut command_encoder, particle_buffer);
        queue.submit(Some(command_encoder.finish()));
    }
}
