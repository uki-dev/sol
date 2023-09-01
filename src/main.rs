use futures::executor::block_on;
use glam::{Quat, Vec3};
use std::time::{Duration, Instant};
use wgpu::{
    DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode,
    RequestAdapterOptions, SurfaceConfiguration, TextureUsages,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod simulation;
use simulation::Simulation;

mod visualisation;
use visualisation::Camera;
use visualisation::Visualisation;

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
                features: Features::empty(),
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

    let mut simulation = Simulation::new(256, 256, 256, &device);
    simulation.populate(&device, &queue);

    let mut distance = 256.;
    let mut camera = Camera::new();
    camera.position = camera.rotation * Vec3::new(0., 0., -distance);
    camera.position.y = -32.;

    let mut visualisation = Visualisation::new(&device, surface_formats.into(), &simulation);

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

                let mut pitch = -(normalized_mouse_y * 2. - 1.) * std::f32::consts::PI;
                let yaw = -(normalized_mouse_x * 2. - 1.) * std::f32::consts::PI;
                pitch = pitch.clamp(0., std::f32::consts::FRAC_PI_2);
                

                camera.rotation = Quat::from_axis_angle(Vec3::Y, yaw)
                    * Quat::from_axis_angle(Vec3::X, pitch);
                camera.position = Vec3::new(0., -0., 0.) + camera.rotation * Vec3::new(0., 0., -distance);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        distance -= y * 8.;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(delta) => {}
                }
                camera.position = Vec3::new(0., -0., 0.) + camera.rotation * Vec3::new(0., 0., -distance);
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
                // limited to 60fps
                let elapsed = last_simulation.elapsed();
                if elapsed >= Duration::from_millis(16) {
                    println!("simulation tick");
                    simulation.simulate(&device, &queue);
                    last_simulation = Instant::now();
                }
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                visualisation.visualise(&device, &queue, &surface, &simulation, &camera);
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
