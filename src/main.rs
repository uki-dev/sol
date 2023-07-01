use futures::executor::block_on;
use std::{borrow::Cow, mem::size_of};
use wgpu::{
    BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, DeviceDescriptor, Features,
    FragmentState, Instance, Limits, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModuleDescriptor, SurfaceConfiguration, TextureUsages,
    TextureViewDescriptor, VertexState,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod engine;
use engine::simulation::Simulation;
use engine::{rendering::Camera, simulation::SimulationDescriptor};

fn main() {
    block_on(async_main());
}

async fn async_main() {
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
    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_formats = surface_capabilities.formats[0];

    let simulation = Simulation::new(
        &SimulationDescriptor {
            width: 8,
            height: 8,
            depth: 8,
        },
        &device,
    );
    simulation.dispatch(&device, &queue);
    let _ = simulation.receive(&device).await;

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("main.wgsl"))),
    });

    let uniform_buffer: wgpu::Buffer = device.create_buffer(&BufferDescriptor {
        label: Some("camera"),
        size: size_of::<f32>() as u64 * 16 * 3,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
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
            targets: &[Some(surface_formats.into())],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
    });

    let mut surface_configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_formats,
        width: 1,
        height: 1,
        present_mode: PresentMode::AutoNoVsync,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    let mut camera = Camera::new();
    camera.position.z = -5.;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                surface_configuration.width = size.width;
                surface_configuration.height = size.height;

                camera.aspect = size.width as f32 / size.height as f32;

                #[repr(C)]
                #[derive(Default, Copy, Clone)]
                struct Uniforms {
                    width: u32,
                    height: u32,
                    depth: u32,
                    _padding: u32,
                    // mat4x4 must be aligned so we add this padding
                    inverse_view_projection: [f32; 4 * 4],
                }

                unsafe impl bytemuck::Pod for Uniforms {}
                unsafe impl bytemuck::Zeroable for Uniforms {}

                let uniforms = Uniforms {
                    width: simulation.width,
                    height: simulation.height,
                    depth: simulation.depth,
                    inverse_view_projection: (camera.projection() * camera.view())
                        .inversed()
                        .as_array()
                        .clone(),
                    ..Default::default()
                };

                queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

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
            // TODO: explicity destroy GPU resources (although many operating systems will do this automatically its not good practice to rely on)
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
