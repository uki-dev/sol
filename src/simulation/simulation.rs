use futures::channel::oneshot;
use std::{borrow::Cow, mem::size_of, process::exit};
// use ultraviolet::Vec3;
use wgpu::{
    BindGroup, Buffer, BufferAsyncError, BufferDescriptor, BufferUsages, ComputePipeline, Device,
    PipelineLayoutDescriptor, Queue, util::{DeviceExt, BufferInitDescriptor},
};

#[path = "../cell.rs"]
mod cell;
use cell::Cell;

#[path = "../object.rs"]
mod object;
use object::Object;

#[repr(C, align(16))]
#[derive(Default, Copy, Clone)]
struct Uniforms {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
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

    uniform_buffer: Buffer,
    pub cells_buffer: Buffer,
    pub objects_buffer: Buffer,
    pub objects_length_buffer: Buffer,
    
    populate_compute_pipeline: ComputePipeline,
    simulate_compute_pipeline: ComputePipeline,
    map_cells_to_objects_compute_pipeline: ComputePipeline,

    bind_group: BindGroup,
}

impl Drop for Simulation {
    fn drop(&mut self) {
        self.uniform_buffer.destroy();
        self.cells_buffer.destroy();
        self.objects_buffer.destroy();
        self.objects_length_buffer.destroy();
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

    pub fn new(width: u32, height: u32, depth: u32, device: &Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("simulation.wgsl"))),
        });

        let uniform_buffer: wgpu::Buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniforms"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[Uniforms {
                width,
                height,
                depth,
            }]),
        });

        let storage_buffer = device.create_buffer(&BufferDescriptor {
            size: size_of::<Cell>() as u64 * (width * height * depth) as u64,
            label: Some("Simulation::storage_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        println!("Object size in bytes {}", size_of::<Object>());
        let objects_buffer = device.create_buffer(&BufferDescriptor {
            size: size_of::<Object>() as u64 * (width * height * depth) as u64,
            label: Some("Simulation::objects_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let objects_length_buffer = device.create_buffer(&BufferDescriptor {
            size: size_of::<u32>() as u64,
            label: Some("Simulation::objects_length_buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
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
                // TODO: move into a separate layout group specific for mapping to objects
                // or abstract this logic from this implementation
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: objects_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: objects_length_buffer.as_entire_binding(),
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

        let map_cells_to_objects_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "map_cells_to_objects",
            });

        Simulation {
            width,
            height,
            depth,
            uniform_buffer,
            cells_buffer: storage_buffer,
            objects_buffer,
            objects_length_buffer,
            populate_compute_pipeline,
            simulate_compute_pipeline,
            map_cells_to_objects_compute_pipeline,
            bind_group,
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
        self.dispatch(device, queue, &self.populate_compute_pipeline);
    }

    pub fn simulate(&mut self, device: &Device, queue: &Queue) {
        self.dispatch(device, queue, &self.simulate_compute_pipeline);
    }

    pub fn map_cells_to_objects(&mut self, device: &Device, queue: &Queue) {
        queue.write_buffer(&self.objects_length_buffer, 0, bytemuck::cast_slice(&[0]));
        self.dispatch(device, queue, &self.map_cells_to_objects_compute_pipeline);
    }
}
