use encase::{internal::WriteInto, ShaderType, UniformBuffer};
use wgpu::{Buffer, Queue};

pub trait QueueUtilities<T: ShaderType + WriteInto> {
    fn write_encased_uniform_buffer(&self, buffer: &Buffer, data: T);
}

impl<T: ShaderType + WriteInto> QueueUtilities<T> for Queue {
    fn write_encased_uniform_buffer(&self, buffer: &Buffer, data: T) {
        let mut encased_uniform_buffer = UniformBuffer::new(Vec::<u8>::new());
        encased_uniform_buffer.write(&data).unwrap();
        self.write_buffer(buffer, 0, &encased_uniform_buffer.into_inner());
    }
}
