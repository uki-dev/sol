use crate::common::{GridCell, GRID_SIZE, MAX_PARTICLES};
use encase::ShaderSize;
use std::borrow::Cow;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, Queue, ShaderModuleDescriptor,
    ShaderSource, ShaderStages,
};

#[include_wgsl_oil::include_wgsl_oil("grid.wgsl")]
mod shader {}

pub struct GridPartition {
    bind_group_layout: BindGroupLayout,
    clear_grid_pipeline: ComputePipeline,
    build_grid_pipeline: ComputePipeline,
    pub grid_buffer: Buffer,
}

impl Drop for GridPartition {
    fn drop(&mut self) {
        self.grid_buffer.destroy();
    }
}

impl GridPartition {
    pub fn new(device: &Device) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(shader::SOURCE)),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: shader::globals::particles::binding::BINDING,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
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
                BindGroupLayoutEntry {
                    binding: shader::globals::grid::binding::BINDING,
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

        let clear_grid_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: shader::entry_points::clear_grid::NAME,
        });

        let build_grid_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: shader::entry_points::build_grid::NAME,
        });

        let grid_buffer = device.create_buffer(&BufferDescriptor {
            size: GridCell::SHADER_SIZE.get() * (GRID_SIZE * GRID_SIZE * GRID_SIZE) as u64,
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        GridPartition {
            bind_group_layout,
            clear_grid_pipeline,
            build_grid_pipeline,
            grid_buffer,
        }
    }

    pub fn build_grid_with_encoder(
        &self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        particle_buffer: &Buffer,
        bounds_buffer: &Buffer,
    ) {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::globals::particles::binding::BINDING,
                    resource: particle_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::globals::bounds::binding::BINDING,
                    resource: bounds_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::globals::grid::binding::BINDING,
                    resource: self.grid_buffer.as_entire_binding(),
                },
            ],
        });

        let mut compute_pass =
            command_encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
        compute_pass.set_bind_group(0, &bind_group, &[]);

        let workgroup_size = shader::entry_points::clear_grid::WORKGROUP_SIZE;
        compute_pass.set_pipeline(&self.clear_grid_pipeline);
        compute_pass.dispatch_workgroups(
            (GRID_SIZE as f32 / workgroup_size[0] as f32).ceil() as u32,
            (GRID_SIZE as f32 / workgroup_size[1] as f32).ceil() as u32,
            (GRID_SIZE as f32 / workgroup_size[2] as f32).ceil() as u32,
        );

        let workgroup_size = shader::entry_points::build_grid::WORKGROUP_SIZE;
        compute_pass.set_pipeline(&self.build_grid_pipeline);
        compute_pass.dispatch_workgroups(
            (MAX_PARTICLES as f32 / workgroup_size[0] as f32).ceil() as u32,
            workgroup_size[1],
            workgroup_size[2],
        );
    }

    pub fn build_grid(
        &self,
        device: &Device,
        queue: &Queue,
        particle_buffer: &Buffer,
        bounds_buffer: &Buffer,
    ) {
        let mut command_encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        self.build_grid_with_encoder(device, &mut command_encoder, particle_buffer, bounds_buffer);
        queue.submit(Some(command_encoder.finish()));
    }
}
