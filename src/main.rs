use futures::executor::block_on;

mod game;

// mod simulation;
// use simulation::Simulation;

// mod visualisation;
// use visualisation::Camera;
// use visualisation::Visualisation;

use wgpu::{Instance, RequestAdapterOptions, PowerPreference, DeviceDescriptor, Features, Limits, SurfaceConfiguration, TextureUsages, PresentMode};
use winit::event::{WindowEvent, Event};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::WindowBuilder;

use crate::game::{System, Game};

fn main() {
    block_on(async_main())
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
    let surface_format = surface_capabilities.formats[0];
    let surface_configuration = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: PresentMode::AutoVsync,
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    // let mut simulation = Simulation::new(256, 256, 256, &device);
    // simulation.populate(&device, &queue);

    // let visualisation = Visualisation::new(
    //     &device,
    //     surface_configuration.format.into(),
    //     &simulation,
    // );

    // let mut camera = Camera::new();
    // camera.position.y = -32.;
    // camera.position.z = -256.;

    struct Application<'a> {
        surface_configuration: &'a SurfaceConfiguration,
    }
    
    impl<'a> Application<'a> {
        pub fn new(surface_configuration: &'a SurfaceConfiguration) -> Application<'a> {
            Application { surface_configuration }
        }
    }
    
    impl<'a> System for Application<'a> {
        fn resize(&mut self, size: (u32, u32)) {
            println!("{}", self.surface_configuration.width);
        }
    }

    let application = Application::new(&surface_configuration);
    let mut game = Game::new();
    game.systems.push(Box::new(application));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => game.resize((size.width, size.height)),
            Event::MainEventsCleared => window.request_redraw(),
            // Event::RedrawRequested(_) => game.update(),
            // TODO: explicity destroy GPU resources (although many operating systems will do this automatically its not good practice to rely on)
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
