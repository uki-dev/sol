use encase::{internal::CreateFrom, ShaderType, StorageBuffer};
use wgpu::{Buffer, Device, Queue};

pub fn debug_buffer<T: ShaderType + CreateFrom>(
    device: &Device,
    queue: &Queue,
    buffer: &Buffer,
) -> T {
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: buffer.size(),
        label: Some("Simulation::staging_buffer"),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut command_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    command_encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, buffer.size());
    queue.submit(core::iter::once(command_encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    device.poll(wgpu::Maintain::Wait);

    let output = buffer_slice.get_mapped_range().to_vec();
    staging_buffer.unmap();

    let result = StorageBuffer::new(output).create().unwrap();
    return result;
}
