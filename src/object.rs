use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct Object {
  matrix: [f32; 4 * 4],
  colour: [f32; 4],
  sdf: u32,
}
unsafe impl Zeroable for Object {}
unsafe impl Pod for Object {}