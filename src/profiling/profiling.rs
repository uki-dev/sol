use core::panic;

use futures::channel::oneshot;
use wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoder, CommandEncoderDescriptor, Device,
    QuerySetDescriptor, QueryType, Queue,
};

pub struct Timing {
    pub start: u64,
    pub end: u64,
}

impl Timing {
    // Duration in ms
    pub fn duration(&self) -> f32 {
        return (self.end - self.start) as f32 / 1000000.0;
    }
}

pub async fn profile<F>(device: &Device, queue: &Queue, f: F) -> Timing
where
    F: Fn(&mut CommandEncoder) -> (),
{
    let query_set = device.create_query_set(&QuerySetDescriptor {
        label: None,
        ty: QueryType::Timestamp,
        count: 2,
    });

    let query_buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        size: 16,
        usage: BufferUsages::QUERY_RESOLVE
            | BufferUsages::STORAGE
            | BufferUsages::COPY_SRC
            | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut command_encoder =
        device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    command_encoder.write_timestamp(&query_set, 0);
    f(&mut command_encoder);
    command_encoder.write_timestamp(&query_set, 1);
    command_encoder.resolve_query_set(&query_set, 0..2, &query_buffer, 0);
    command_encoder.copy_buffer_to_buffer(&query_buffer, 0, &staging_buffer, 0, 16);
    queue.submit(Some(command_encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = oneshot::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, |v| sender.send(v).unwrap());
    device.poll(wgpu::Maintain::Wait);
    if let Ok(Ok(())) = receiver.await {
        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice::<u8, u64>(&data);
        query_buffer.destroy();
        staging_buffer.destroy();
        return Timing {
            start: result[0],
            end: result[1],
        };
    } else {
        panic!("Failed to profile");
    }
}
