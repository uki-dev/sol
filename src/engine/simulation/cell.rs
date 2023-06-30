use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    // TODO: wgsl does not support u8 so until we work out how to decode bytes into `pub material: Material` correctly, simply use this
    material: u32,
    // TODO: having some issues with this not matching wgsl size, might not be using `f32` internally ? the lib is crazy macro heavy which makes it super hard to dig into
    // pub velocity: Vec3,
}
impl Cell {
    // pub fn material(&self) -> Material {
    //     Material::from(self.material)
    // }
}
unsafe impl Zeroable for Cell {}
unsafe impl Pod for Cell {}
