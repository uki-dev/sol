use std::{borrow::Cow, fs};

use wgpu::{
    util::DeviceExt, Color, CommandEncoderDescriptor, DeviceDescriptor, Features, FragmentState,
    Instance, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PowerPreference, PresentMode, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
    SurfaceConfiguration, TextureUsages, TextureViewDescriptor, VertexState,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod engine;
use engine::rendering::Camera;

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("🌎")
        .build(&event_loop)
        .unwrap();

    let instance = Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to request adapter");

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: Limits::downlevel_defaults().using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to request device");

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Borrowed(&fs::read_to_string("src/main.wgsl").unwrap())),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_formats = surface_capabilities.formats[0];

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
            targets: &[Some(surface_formats.into())],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    });

    let size = window.inner_size();
    let mut surface_configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_formats,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::AutoNoVsync,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &surface_configuration);

    let camera = Camera::new();
    let inverse_view_projection = (camera.view() * camera.projection()).inversed();

    let mut data = [0f32; 16 * 3];
    data[..16].copy_from_slice(camera.view().as_array());
    data[16..32].copy_from_slice(camera.projection().as_array());
    data[32..48].copy_from_slice(inverse_view_projection.as_array());

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("camera"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: None,
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                surface_configuration.width = size.width;
                surface_configuration.height = size.height;
                surface.configure(&device, &surface_configuration);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let current_texture = surface
                    .get_current_texture()
                    .expect("Failed to get current texture");

                let view = current_texture
                    .texture
                    .create_view(&TextureViewDescriptor::default());

                let mut command_encoder =
                    device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                {
                    let mut render_pass =
                        command_encoder.begin_render_pass(&RenderPassDescriptor {
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
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.draw(0..6, 0..1);
                }

                queue.submit(Some(command_encoder.finish()));
                current_texture.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
