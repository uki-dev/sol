use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    // TODO: wgsl does not support u8 so until we work out how to decode bytes into `pub material: Material` correctly, simply use this
    material: u32,
    // TODO: need to add padding so vec3 aligns
    // pub velocity: Vec3,
}
impl Cell {
    // pub fn material(&self) -> Material {
    //     Material::from(self.material)
    // }
}
unsafe impl Zeroable for Cell {}
unsafe impl Pod for Cell {}
