use std::{borrow::Cow::Borrowed, mem::size_of};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    Color, ColorTargetState, CommandEncoderDescriptor, Device, FragmentState, LoadOp,
    MultisampleState, Operations, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, TextureViewDescriptor,
    VertexState,
};

use super::Camera;
use crate::simulation::Simulation;

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Uniforms {
    width: u32,
    height: u32,
    depth: u32,
    _padding_0: u32,
    camera_position: [f32; 3],
    _padding_1: f32,
    inverse_view_projection: [f32; 4 * 4],
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

pub struct Visualisation {
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

struct Dimensions {
    width: u32,
    height: u32,
}

impl Visualisation {
    pub fn new(device: &Device, target: ColorTargetState, simulation: &Simulation) -> Self {
        let (uniform_buffer, bind_group, render_pipeline) =
            Self::initialise(device, target, simulation);
        Visualisation {
            uniform_buffer,
            bind_group,
            render_pipeline,
        }
    }

    fn initialise(
        device: &Device,
        target: ColorTargetState,
        simulation: &Simulation,
    ) -> (Buffer, BindGroup, RenderPipeline) {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Borrowed(include_str!("visualisation.wgsl"))),
        });

        let uniform_buffer: Buffer = device.create_buffer(&BufferDescriptor {
            label: Some("uniforms"),
            size: size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
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
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: simulation.storage_buffer.as_entire_binding(),
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

        return (uniform_buffer, bind_group, render_pipeline);
    }

    pub fn visualise(
        &self,
        device: &Device,
        queue: &Queue,
        surface: &Surface,
        simulation: &Simulation,
        camera: &Camera,
    ) {
        let (uniform_buffer, bind_group, render_pipeline) = (
            &self.uniform_buffer,
            &self.bind_group,
            &self.render_pipeline,
        );

        let uniforms = Uniforms {
            width: simulation.width(),
            height: simulation.height(),
            depth: simulation.depth(),
            camera_position: camera.position.to_array(),
            inverse_view_projection: (camera.projection() * camera.view())
                .inverse()
                .to_cols_array()
                .clone(),
            ..Default::default()
        };
        queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let current_texture = surface
            .get_current_texture()
            .expect("Failed to get current texture");

        let view = current_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

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
        current_texture.present();
    }
}
