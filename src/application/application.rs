use wgpu::{
    Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::events::Listeners;

pub struct Application {
    event_loop: EventLoop<()>,
    window: Window,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub surface_configuration: SurfaceConfiguration,
    pub listeners: Listeners,
}

impl Application {
    pub async fn new() -> Self {
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

        let listeners = Listeners::default();

        Application {
            event_loop,
            window,
            device,
            queue,
            surface,
            surface_configuration,
            listeners,
        }
    }

    pub fn run(mut self) {
        self.listeners.resize.add(move |&(width, height)| {
            self.surface_configuration.width = width;
            self.surface_configuration.height = height;
            self.surface
                .configure(&self.device, &self.surface_configuration);
        });

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => self.listeners.resize.emit((size.width, size.height)),
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => self.listeners.draw.emit(()),
                // TODO: explicity destroy GPU resources (although many operating systems will do this automatically its not good practice to rely on)
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == self.window.id() => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }
}
