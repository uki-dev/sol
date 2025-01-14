use futures::executor::block_on;
use glam::{IVec3, Quat, Vec3};
use std::time::{Duration, Instant};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode,
    RequestAdapterOptions, SurfaceConfiguration, TextureUsages, TextureViewDescriptor,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use encase::StorageBuffer;

pub mod common;
use crate::common::{Bounds, GridCell, Particle, MAX_PARTICLES};

mod partition;
use crate::partition::{BoundsPartition, GridPartition};

mod visualisation;
use crate::visualisation::{Camera, Visualisation};

mod simulation;
use crate::simulation::Simulation;

pub mod debug;
use debug::debug_buffer;

pub mod profiling;
use crate::profiling::profile;

pub mod wgpu_utilities;

use rand::Rng;

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

    let simulation = Simulation::new(&device);

    let mut rng = rand::thread_rng();
    let mut particles = vec![
        Particle {
            position: Vec3::new(0.0, 0.0, 0.0),
            old_position: Vec3::new(0.0, 0.0, 0.0)
        };
        MAX_PARTICLES as usize
    ];

    for particle in particles.iter_mut() {
        let position = Vec3::new(
            rng.gen_range(-16.0..16.0),
            rng.gen_range(-16.0..16.0),
            rng.gen_range(-16.0..16.0),
        );
        particle.position = position;
        particle.old_position = position;
    }

    let mut encased_particle_buffer = StorageBuffer::new(Vec::<u8>::new());
    encased_particle_buffer.write(&particles).unwrap();
    let data = encased_particle_buffer.into_inner();
    queue.write_buffer(&simulation.particle_buffer, 0, &data);

    // let data = debug_buffer::<Vec<Particle>>(&device, &queue, &particle_buffer);
    // println!("Particles {:?}", data);

    let bounds_partition = BoundsPartition::new(&device);
    let timing = profile(&device, &queue, |command_encoder| {
        bounds_partition.calculate_bounds_with_encoder(
            &device,
            &queue,
            command_encoder,
            &simulation.particle_buffer,
        );
    })
    .await;
    // TODO: We should just rename this to some read buffer utility and then print it on the consumer side
    let data = debug_buffer::<Bounds>(&device, &queue, &bounds_partition.bounds_buffer);
    println!("Bounds: {:?}", data);
    println!("Calculate bounds duration: {}ms", timing.duration());

    let grid_partition = GridPartition::new(&device);
    let timing = profile(&device, &queue, |command_encoder| {
        grid_partition.build_grid_with_encoder(
            &device,
            command_encoder,
            &simulation.particle_buffer,
            &bounds_partition.bounds_buffer,
        );
    })
    .await;
    // TODO: We should just rename this to some read buffer utility and then print it on the consumer side
    // let data = debug_buffer::<Vec<GridCell>>(&device, &queue, &grid_partition.grid_buffer);
    // println!("Grid: {:?}", data);
    // let total_grid_particles: u32 = data.iter().map(|element| element.particles_length).sum();
    // println!(
    //     "Max Particles {}, Grid Particles {}",
    //     MAX_PARTICLES, total_grid_particles
    // );
    println!("Build grid duration: {}ms", timing.duration());

    // let mut simulation = Simulation::new(8, 8, 8, &device);
    // simulation.populate(&device, &queue);
    let mut distance = 64.;
    let mut camera = Camera::new();
    camera.position = camera.rotation * Vec3::new(0., 0., -distance);

    let visualisation = Visualisation::new(&device, surface_formats.into());

    let mut is_focused = true;
    let mut frame_count = 0;

    let start_instant = Instant::now();
    let mut last_frame_time = start_instant;
    let mut previous_instant = start_instant;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Focused(focused),
                ..
            } => {
                is_focused = focused;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let window_size = window.inner_size();

                let normalized_mouse_x = position.x as f32 / window_size.width as f32;
                let normalized_mouse_y = position.y as f32 / window_size.height as f32;

                let mut pitch = -(normalized_mouse_y * 2. - 1.) * std::f32::consts::PI;
                let yaw = -(normalized_mouse_x * 2. - 1.) * std::f32::consts::PI;
                pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

                camera.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
                camera.position =
                    Vec3::new(0., -0., 0.) + camera.rotation * Vec3::new(0., 0., -distance);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        distance -= y * 8.;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(_) => {}
                }
                camera.position =
                    Vec3::new(0., -0., 0.) + camera.rotation * Vec3::new(0., 0., -distance);
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                camera.aspect = size.width as f32 / size.height as f32;
                surface_configuration.width = size.width;
                surface_configuration.height = size.height;
                surface.configure(&device, &surface_configuration);
            }
            Event::MainEventsCleared => {
                let instant = Instant::now();
                let time = start_instant.elapsed().as_secs_f32();
                let delta_time = instant.duration_since(previous_instant).as_secs_f32();
                frame_count += 1;

                let elapsed = last_frame_time.elapsed();
                if elapsed >= Duration::from_millis(1000) {
                    let fps = frame_count as f32 / elapsed.as_secs_f32();
                    let delta_time = elapsed.as_secs_f32() / frame_count as f32 * 1000.0;
                    let title = format!("🌎 | {:.0}fps | {:.2}ms", fps, delta_time);
                    window.set_title(&title);
                    last_frame_time = instant;
                    frame_count = 0;
                }

                previous_instant = instant;

                if (!is_focused) {
                    return;
                }

                let gravity: Vec3 = Vec3::new(0.0, -9.8, 0.0);
                let spin_rate = std::f32::consts::PI / 32.0;
                let gravity_rotation = Quat::from_euler(
                    glam::EulerRot::XYZ,
                    spin_rate * time,
                    spin_rate * time,
                    spin_rate * time,
                );
                let rotated_gravity = gravity_rotation * gravity;

                simulation.simulate(
                    &device,
                    &queue,
                    &bounds_partition.bounds_buffer,
                    &grid_partition.grid_buffer,
                    delta_time,
                    rotated_gravity,
                );

                // // TODO: `build_grid` is not stable and seems to produce different data even with the same input
                grid_partition.build_grid(
                    &device,
                    &queue,
                    &simulation.particle_buffer,
                    &bounds_partition.bounds_buffer,
                );

                let current_texture = surface
                    .get_current_texture()
                    .expect("Failed to get current texture");
                let view = current_texture
                    .texture
                    .create_view(&TextureViewDescriptor::default());
                visualisation.visualise(
                    &device,
                    &queue,
                    &view,
                    &simulation.particle_buffer,
                    &bounds_partition.bounds_buffer,
                    &grid_partition.grid_buffer,
                    &camera,
                );
                current_texture.present();

                window.request_redraw();
            }
            // TODO: explicity destroy GPU resources
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
