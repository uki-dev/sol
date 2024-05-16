use crate::Camera;
use bytemuck::{Pod, Zeroable};
use encase::{ShaderSize, UniformBuffer};
use std::borrow::Cow::Borrowed;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    Color, ColorTargetState, CommandEncoderDescriptor, Device, FragmentState, LoadOp,
    MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource::Wgsl, ShaderStages, TextureView, VertexState,
};

#[include_wgsl_oil::include_wgsl_oil("visualisation.wgsl")]
mod visualisation_shader {}
pub use visualisation_shader::types::Uniforms;
unsafe impl Pod for Uniforms {}
unsafe impl Zeroable for Uniforms {}
impl Copy for Uniforms {}

pub struct Visualisation {
    bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,
    uniform_buffer: Buffer,
}

impl Visualisation {
    pub fn new(device: &Device, target: ColorTargetState) -> Self {
        let (bind_group_layout, render_pipeline, uniform_buffer) = Self::initialise(device, target);
        Visualisation {
            bind_group_layout,
            render_pipeline,
            uniform_buffer,
        }
    }

    fn initialise(
        device: &Device,
        target: ColorTargetState,
    ) -> (BindGroupLayout, RenderPipeline, Buffer) {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: Wgsl(Borrowed(visualisation_shader::SOURCE)),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Uniforms
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Particles
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Bounds
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Grid
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
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

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fragment",
                targets: &[Some(target)],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let uniform_buffer: Buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: Uniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        return (bind_group_layout, render_pipeline, uniform_buffer);
    }

    pub fn visualise(
        &self,
        device: &Device,
        queue: &Queue,
        view: &TextureView,
        particle_buffer: &Buffer,
        bounds_buffer: &Buffer,
        grid_buffer: &Buffer,
        camera: &Camera,
    ) {
        let (uniform_buffer, bind_group_layout, render_pipeline) = (
            &self.uniform_buffer,
            &self.bind_group_layout,
            &self.render_pipeline,
        );

        let uniforms = Uniforms {
            camera_position: camera.position,
            inverse_view_projection: (camera.projection() * camera.view()).inverse(),
        };

        let mut encased_uniform_buffer = UniformBuffer::new(Vec::<u8>::new());
        encased_uniform_buffer.write(&uniforms).unwrap();
        queue.write_buffer(&uniform_buffer, 0, &encased_uniform_buffer.into_inner());

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: bounds_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: grid_buffer.as_entire_binding(),
                },
            ],
        });

        let mut command_encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        queue.submit(Some(command_encoder.finish()));
    }
}
