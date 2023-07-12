use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
// use futures::executor::block_on;
use std::time::{Duration, Instant};

mod application;
use application::Application;

mod simulation;
use futures::executor::block_on;
use simulation::Simulation;

mod visualisation;
use visualisation::Camera;
use visualisation::Visualisation;

fn main() {
    block_on(async_main())
}

async fn async_main() {
    let mut application = Application::new().await;

    let mut simulation = Simulation::new(256, 256, 256, &application.device);
    simulation.populate(&application.device, &application.queue);

    let visualisation = Visualisation::new(
        &application.device,
        application.surface_configuration.format.into(),
        &simulation,
    );

    let mut camera = Camera::new();
    camera.position.y = -32.;
    camera.position.z = -256.;

    let listeners = application.listeners;
    listeners
        .resize
        .add(|&(width, height)| camera.aspect = width as f32 / height as f32);

    let mut last_update = Instant::now();
    listeners.draw.add(|()| {
        if last_update.elapsed() >= Duration::from_millis(16) {
            simulation.simulate(&application.device, &application.queue);
        }
        visualisation.visualise(
            &application.device,
            &application.queue,
            &application.surface,
            &simulation.borrow(),
            &camera,
        )
    });
    application.run();
}
