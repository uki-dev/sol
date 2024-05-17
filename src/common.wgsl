const GRID_SIZE = 2u;
const MAX_PARTICLES = 16u; 
const MAX_PARTICLES_PER_GRID_CELL = MAX_PARTICLES / GRID_SIZE;

@export struct Particle {
  position: vec3<f32>,
}

@export struct Bounds {
  min_x: atomic<i32>,
  min_y: atomic<i32>,
  min_z: atomic<i32>,
  max_x: atomic<i32>,
  max_y: atomic<i32>,
  max_z: atomic<i32>,
}

@export struct GridCell {
  particles: array<u32, MAX_PARTICLES_PER_GRID_CELL>,
  particles_length: atomic<u32>,
}

fn grid_position_to_grid_index(position: vec3<i32>) -> i32 {
  let grid_size = i32(GRID_SIZE);
  return position.x + position.y * grid_size + position.z * grid_size * grid_size;
}

fn world_position_to_grid_position(position: vec3<f32>, bounds_min: vec3<f32>, bounds_max: vec3<f32>) -> vec3<i32> {
  return vec3<i32>(round((position - bounds_min) / (bounds_max - bounds_min) * f32(GRID_SIZE - 1)));
}

fn world_position_to_grid_index(position: vec3<f32>, bounds_min: vec3<f32>, bounds_max: vec3<f32>) -> i32 {
  return grid_position_to_grid_index(world_position_to_grid_position(position, bounds_min, bounds_max));
}