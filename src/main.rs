use futures::executor::block_on;
use glam::{Quat, Vec3};
use std::{
    mem::size_of,
    time::{Duration, Instant},
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferDescriptor, BufferUsages, DeviceDescriptor, Features, Instance, Limits, PowerPreference,
    PresentMode, RequestAdapterOptions, SurfaceConfiguration, TextureUsages, TextureViewDescriptor,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod data;
use data::Particle;

mod partition;

use crate::partition::GridCell;
use crate::partition::SpatialPartioner;
use crate::partition::GRID_SIZE;

mod simulation;
use simulation::Simulation;

mod visualisation;
use visualisation::Camera;
use visualisation::Visualisation;

pub mod debug;
use debug::debug_buffer;

pub mod profiling;
use crate::profiling::profile;

use crate::data::Bounds;

fn main() {
    block_on(async_main());
}

async fn async_main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("🌎")
        // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
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
                features: Features::TIMESTAMP_QUERY,
                limits: Limits::downlevel_defaults().using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to request device");
    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_formats = surface_capabilities.formats[0];
    let mut surface_configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_formats,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: PresentMode::AutoVsync,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    let mut particles = vec![
        Particle {
            position: [0.0, 0.0, 0.0, 0.0],
        };
        3
    ];

    particles[0] = Particle {
        position: [16.0, 32.0, 64.0, 0.0],
    };

    particles[1] = Particle {
        position: [-64.0, -32.0, -16.0, 0.0],
    };

    particles[2] = Particle {
        position: [-16.0, -16.0, 32.0, 0.0],
    };

    let particle_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("main::particle_buffer"),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        contents: bytemuck::cast_slice(&particles),
    });
    let _ =
        debug_buffer::<Particle>(&device, &queue, &particle_buffer, particles.len() as u64).await;

    let spatial_partitioner = SpatialPartioner::new(&device, &particle_buffer, particles.len());
    let timing = profile(&device, &queue, |command_encoder| {
        spatial_partitioner.compute_bounds_with_encoder(&queue, command_encoder);
    })
    .await;
    // TODO: We should just rename this to some read buffer utility and then print it on the consumer side
    let _ = debug_buffer::<Bounds>(&device, &queue, &spatial_partitioner.bounds_buffer, 1).await;
    println!("Compute bounds duration: {}ms", timing.duration());

    let timing = profile(&device, &queue, |command_encoder| {
        spatial_partitioner.compute_grid_with_encoder(command_encoder);
    })
    .await;
    // TODO: We should just rename this to some read buffer utility and then print it on the consumer side
    let _ = debug_buffer::<GridCell>(
        &device,
        &queue,
        &spatial_partitioner.grid_buffer,
        (GRID_SIZE * GRID_SIZE * GRID_SIZE) as u64,
    )
    .await;
    println!("Compute grid duration: {}ms", timing.duration());
    // let mut simulation = Simulation::new(8, 8, 8, &device);
    // simulation.populate(&device, &queue);

    // let mut distance = 16.;
    // let mut camera = Camera::new();
    // camera.position = camera.rotation * Vec3::new(0., 0., -distance);

    // let visualisation = Visualisation::new(
    //     &device,
    //     surface_formats.into(),
    //     &simulation.objects_buffer,
    //     &simulation.objects_length_buffer,
    // );

    // let mut last_tick = Instant::now();
    // event_loop.run(move |event, _, control_flow| {
    //     *control_flow = ControlFlow::Poll;
    //     match event {
    //         Event::WindowEvent {
    //             event: WindowEvent::CursorMoved { position, .. },
    //             ..
    //         } => {
    //             let window_size = window.inner_size();

    //             let normalized_mouse_x = position.x as f32 / window_size.width as f32;
    //             let normalized_mouse_y = position.y as f32 / window_size.height as f32;

    //             let mut pitch = -(normalized_mouse_y * 2. - 1.) * std::f32::consts::PI;
    //             let yaw = -(normalized_mouse_x * 2. - 1.) * std::f32::consts::PI;
    //             pitch = pitch.clamp(0., std::f32::consts::FRAC_PI_2);

    //             camera.rotation =
    //                 Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
    //             camera.position =
    //                 Vec3::new(0., -0., 0.) + camera.rotation * Vec3::new(0., 0., -distance);
    //         }
    //         Event::WindowEvent {
    //             event: WindowEvent::MouseWheel { delta, .. },
    //             ..
    //         } => {
    //             match delta {
    //                 winit::event::MouseScrollDelta::LineDelta(_, y) => {
    //                     distance -= y * 8.;
    //                 }
    //                 winit::event::MouseScrollDelta::PixelDelta(delta) => {}
    //             }
    //             camera.position =
    //                 Vec3::new(0., -0., 0.) + camera.rotation * Vec3::new(0., 0., -distance);
    //         }
    //         Event::WindowEvent {
    //             event: WindowEvent::Resized(size),
    //             ..
    //         } => {
    //             camera.aspect = size.width as f32 / size.height as f32;
    //             surface_configuration.width = size.width;
    //             surface_configuration.height = size.height;
    //             surface.configure(&device, &surface_configuration);
    //         }
    //         Event::MainEventsCleared => {
    //             let elapsed = last_tick.elapsed();
    //             if elapsed >= Duration::from_millis(100) {
    //                 simulation.simulate(&device, &queue);
    //                 simulation.map_cells_to_objects(&device, &queue);
    //                 last_tick = Instant::now();
    //             }
    //             window.request_redraw();
    //         }
    //         Event::RedrawRequested(_) => {
    //             let current_texture = surface
    //                 .get_current_texture()
    //                 .expect("Failed to get current texture");
    //             let view = current_texture
    //                 .texture
    //                 .create_view(&TextureViewDescriptor::default());
    //             visualisation.visualise(&device, &queue, &view, &camera);
    //             current_texture.present();
    //         }
    //         // TODO: explicity destroy GPU resources (although many operating systems will do this automatically its not good practice to rely on)
    //         Event::WindowEvent {
    //             event: WindowEvent::CloseRequested,
    //             window_id,
    //         } if window_id == window.id() => *control_flow = ControlFlow::Exit,
    //         _ => (),
    //     }
    // });
}
