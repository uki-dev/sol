#import ../common.wgsl as Common

@export struct AtomicGridCell {
  particles: array<u32, Common::MAX_PARTICLES_PER_GRID_CELL>,
  particles_length: atomic<u32>,
}

@group(0)
@binding(0)
var<storage, read> particles: array<Common::Particle>;

@group(0)
@binding(1)
var<storage, read> bounds: Common::Bounds;

@group(0)
@binding(21)
var<storage, read_write> grid: array<AtomicGridCell>;

@compute
@workgroup_size(4, 4, 4)
fn clear_grid(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let grid_position = vec3<i32>(global_invocation_id);
  let grid_index = Common::grid_position_to_grid_index(grid_position);
  grid[grid_index].particles_length = 0u;
}

@compute
@workgroup_size(1)
fn build_grid(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let particle_index = global_invocation_id.x;
  let particle = particles[particle_index];
  let grid_index = Common::world_position_to_grid_index(particles[particle_index].position, bounds);
  let particles_length = atomicAdd(&grid[grid_index].particles_length, 1u);
  grid[grid_index].particles[particle_index] = particle_index;
}