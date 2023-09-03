use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    material: u32,
}
unsafe impl Zeroable for Cell {}
unsafe impl Pod for Cell {}
