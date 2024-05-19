const GRID_SIZE = 16u;
const MAX_PARTICLES = 2048u; 
const MAX_PARTICLES_PER_GRID_CELL = MAX_PARTICLES / GRID_SIZE;
const PARTICLE_RADIUS = 0.8;

@export struct Particle {
  position: vec3<f32>,
  old_position: vec3<f32>,
}

@export struct Bounds {
  min_x: i32,
  min_y: i32,
  min_z: i32,
  max_x: i32,
  max_y: i32,
  max_z: i32,
}

@export struct GridCell {
  particles: array<u32, MAX_PARTICLES_PER_GRID_CELL>,
  particles_length: u32,
}

fn grid_position_to_grid_index(position: vec3<i32>) -> i32 {
  let grid_size = i32(GRID_SIZE);
  return position.x + position.y * grid_size + position.z * grid_size * grid_size;
}

fn world_position_to_grid_position(position: vec3<f32>, bounds: Bounds) -> vec3<i32> {
  let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
  let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));
  return vec3<i32>(round((position - bounds_min) / (bounds_max - bounds_min) * f32(GRID_SIZE - 1)));
}

fn world_position_to_grid_index(position: vec3<f32>, bounds: Bounds) -> i32 {
  return grid_position_to_grid_index(world_position_to_grid_position(position, bounds));
}