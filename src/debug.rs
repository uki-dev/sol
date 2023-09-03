use std::{mem::size_of, fmt::Debug};

use bytemuck::Pod;
use futures::channel::oneshot;
use wgpu::{Device, Queue, Buffer, BufferAsyncError};

// TODO: can we infer the array type and calculate size from that without passing length?
pub async fn debug_buffer<T: Debug + Pod>(device: &Device, queue: &Queue, buffer: &Buffer, length: u64) -> Result<(), BufferAsyncError> {
  let size = size_of::<T>() as u64 * length;
  let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      size,
      label: Some("Simulation::staging_buffer"),
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
  });

  let mut encoder =
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
  encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, size);
  queue.submit(Some(encoder.finish()));

  let buffer_slice = staging_buffer.slice(..);
  let (sender, receiver) = oneshot::channel();
  buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
  device.poll(wgpu::Maintain::Wait);
  match &receiver.await {
      Ok(Err(error)) => return Err(error.clone()),
      // TODO: should we throw error if buffer mapping is cancelled ?
      _ => {}
  };
  let data = buffer_slice.get_mapped_range();
  let result = bytemuck::cast_slice::<u8, T>(&data);
  staging_buffer.destroy();

  for (i, element) in result.iter().enumerate() {
      println!("element: {:?}", element);
  }

  Ok(())
}