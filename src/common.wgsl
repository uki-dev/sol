const GRID_SIZE = 2u;
const MAX_PARTICLES = 3u;
const MAX_PARTICLES_PER_GRID_CELL = 4u;

@export struct Particle {
  position: vec3<f32>,
}

@export struct Bounds {
  min: vec3<i32>,
  max: vec3<i32>
}

@export struct GridCell {
  particles: array<u32, MAX_PARTICLES_PER_GRID_CELL>,
  particles_length: u32,
}

fn grid_position_to_grid_index(position: vec3<i32>) -> i32 {
  let grid_size = i32(GRID_SIZE);
  return position.x + position.y * grid_size + position.z * grid_size * grid_size;
}


fn world_position_to_grid_index(position: vec3<f32>, bounds: Bounds) -> i32 {
  let bounds_min = vec3<f32>(vec3<i32>(bounds.min.x, bounds.min.y, bounds.min.z));
  let bounds_max = vec3<f32>(vec3<i32>(bounds.max.x, bounds.max.y, bounds.max.z));
  let grid_position = vec3<i32>(round((position - bounds_min) / (bounds_max - bounds_min) * f32(GRID_SIZE - 1)));
  return grid_position_to_grid_index(grid_position);
}