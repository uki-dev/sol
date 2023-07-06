use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    material: u32,
}
impl Cell {
    // pub fn material(&self) -> Material {
    //     Material::from(self.material)
    // }
}
unsafe impl Zeroable for Cell {}
unsafe impl Pod for Cell {}
