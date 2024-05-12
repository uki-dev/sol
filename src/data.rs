use bytemuck::{NoUninit, Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min: [i32; 3],
    pub max: [i32; 3], // xyz + 1 padding for memory alignment
}
unsafe impl Zeroable for Bounds {}
unsafe impl Pod for Bounds {}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub position: [f32; 4], // xyz + 1 padding for memory alignment
}
unsafe impl Zeroable for Particle {}
unsafe impl Pod for Particle {}

#[repr(C, align(16))]
#[derive(Debug)]
pub struct Octree {
    bounds: Bounds,
    particles: [Particle],
}
