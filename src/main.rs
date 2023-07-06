use futures::executor::block_on;
use glam::{Quat, Vec3};
use std::{
    borrow::Cow,
    mem::size_of,
    num::NonZeroU64,
    time::{Duration, Instant},
};
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
use engine::rendering::Camera;
use engine::simulation::Simulation;

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

    let mut simulation = Simulation::new(32, 32, 32, &device);
    simulation.populate(&device, &queue);

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("main.wgsl"))),
    });

    let uniform_buffer: wgpu::Buffer = device.create_buffer(&BufferDescriptor {
        label: Some("uniforms"),
        size: size_of::<Uniforms>() as u64,
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

    let mut distance = 38.;
    let mut camera = Camera::new();
    camera.position = camera.rotation * Vec3::new(0., 0., -distance);

    let mut last_simulation = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let window_size = window.inner_size();

                let normalized_mouse_x = position.x as f32 / window_size.width as f32;
                let normalized_mouse_y = position.y as f32 / window_size.height as f32;

                let rotation_x = (normalized_mouse_y * 2. - 1.) * std::f32::consts::PI;
                let rotation_y = -(normalized_mouse_x * 2. - 1.) * std::f32::consts::PI;

                // camera.rotation = Quat::from_axis_angle(Vec3::X, rotation_x)
                //     * Quat::from_axis_angle(Vec3::Y, rotation_y);
                // camera.position = camera.rotation * Vec3::new(0., 0., -distance);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        distance -= y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {}
                }
                camera.position = camera.rotation * Vec3::new(0., 0., -distance);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                surface_configuration.width = size.width;
                surface_configuration.height = size.height;

                camera.aspect = size.width as f32 / size.height as f32;

                surface.configure(&device, &surface_configuration);
            }
            Event::MainEventsCleared => {
                let elapsed = last_simulation.elapsed();
                if elapsed >= Duration::from_millis(200) {
                    println!("simulation tick");
                    simulation.simulate(&device, &queue);
                    last_simulation = Instant::now();
                }
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
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
