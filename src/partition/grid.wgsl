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
@binding(2)
var<storage, read_write> grid: array<AtomicGridCell>;

@compute
@workgroup_size(1, 1, 1)
fn clear_grid(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let grid_position = vec3<i32>(global_invocation_id);
  let grid_index = Common::grid_position_to_grid_index(grid_position);
  grid[grid_index].particles_length = 0u;
}

// TODO: Replace this with actual particle radius
const PARTICLE_RADIUS = 0.5;

@compute
@workgroup_size(1)
fn build_grid(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let particle_index = global_invocation_id.x;
  let particle = particles[particle_index];
  let bounds_min = vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z);
  let bounds_max = vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z);
  // TODO: Not sure why we need the `* 4.0` here, but it seems to ensure that a particle is populated in all influenced cells
  let min_grid_position = clamp(Common::world_position_to_grid_position(particle.position - vec3<f32>(PARTICLE_RADIUS * 2.0), bounds), bounds_min, bounds_max);
  let max_grid_position = clamp(Common::world_position_to_grid_position(particle.position + vec3<f32>(PARTICLE_RADIUS * 2.0), bounds), bounds_min, bounds_max);
  var grid_position = vec3<i32>();
  for (grid_position.x = min_grid_position.x; grid_position.x <= max_grid_position.x; grid_position.x++) {
    for (grid_position.y = min_grid_position.y; grid_position.y <= max_grid_position.y; grid_position.y++) {
      for (grid_position.z = min_grid_position.z; grid_position.z <= max_grid_position.z; grid_position.z++) {
          let grid_index = Common::grid_position_to_grid_index(grid_position);
          let particles_length = atomicAdd(&grid[grid_index].particles_length, 1u);
          grid[grid_index].particles[particles_length] = particle_index;
      }
    }
  }
}