#import ../common.wgsl as Common

@group(0)
@binding(0)
var<storage, read> particles: array<Common::Particle>;

@group(0)
@binding(1)
var<storage, read> bounds: Common::Bounds;

@group(0)
@binding(2)
var<storage, read_write> grid: array<Common::GridCell>;

@compute
@workgroup_size(1, 1, 1)
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
  let bounds_min = vec3<f32>(vec3<i32>(
      atomicLoad(&bounds.min_x),
      atomicLoad(&bounds.min_y),
      atomicLoad(&bounds.min_z)
  ));
  let bounds_max = vec3<f32>(vec3<i32>(
      atomicLoad(&bounds.max_x),
      atomicLoad(&bounds.max_y),
      atomicLoad(&bounds.max_z)
  ));
  let grid_index = Common::world_position_to_grid_index(particles[particle_index].position, bounds_min, bounds_max);
  let particles_length = atomicAdd(&grid[grid_index].particles_length, 1u);
  grid[grid_index].particles[particles_length] = particle_index;
}