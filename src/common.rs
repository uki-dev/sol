use bytemuck::{Pod, Zeroable};

#[include_wgsl_oil::include_wgsl_oil("common.wgsl")]
pub mod common {}

pub use common::constants::GRID_SIZE::VALUE as GRID_SIZE;
pub use common::constants::MAX_PARTICLES::VALUE as MAX_PARTICLES;
pub use common::constants::MAX_PARTICLES_PER_GRID_CELL::VALUE as MAX_PARTICLES_PER_GRID_CELL;

pub use common::types::Particle;
unsafe impl Pod for Particle {}
unsafe impl Zeroable for Particle {}
impl Copy for Particle {}

pub use common::types::Bounds;
unsafe impl Pod for Bounds {}
unsafe impl Zeroable for Bounds {}
impl Copy for Bounds {}

pub use common::types::GridCell;
unsafe impl Pod for GridCell {}
unsafe impl Zeroable for GridCell {}
impl Copy for GridCell {}
