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

const GRID_SIZE = 2u;
const MAX_PARTICLES_PER_GRID_CELL = 4u;

@export struct GridCell {
  particles: array<u32, MAX_PARTICLES_PER_GRID_CELL>,
  particles_length: atomic<i32>,
}

// const MAX_PARTICLES_PER_NODE = 10000;

// struct OctreeNode {
//   bounds: Bounds,
//   children: array<u32, 8>,
//   particles: array<u32, MAX_PARTICLES_PER_NODE>,
//   particles_length: atomic<u32>,
// }

@group(0)
@binding(0)
var<storage, read> particles: array<Particle>;

@group(0)
@binding(1)
var<storage, read_write> bounds: Bounds;

@group(0)
@binding(2)
var<storage, read_write> grid: array<GridCell>;

// @group(0)
// @binding(2)
// var<storage, read_write> octree_nodes: array<OctreeNode>;

// @group(0)
// @binding(3)
// var<storage, read_write> octree_nodes_length: atomic<u32>;

@compute
@workgroup_size(256)
fn compute_bounds(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let particle_index = global_invocation_id.x;
  let particle = particles[particle_index];
  atomicMin(&bounds.min_x, i32(floor(particle.position.x)));
  atomicMin(&bounds.min_y, i32(floor(particle.position.y)));
  atomicMin(&bounds.min_z, i32(floor(particle.position.z)));
  atomicMax(&bounds.max_x, i32(ceil(particle.position.x)));
  atomicMax(&bounds.max_y, i32(ceil(particle.position.y)));
  atomicMax(&bounds.max_z, i32(ceil(particle.position.z)));
}

// This approach seems great as it allows for us to use encoding, but it does not seem to sync properly as after loading, the minimum value could have changed
// @compute
// @workgroup_size(64)
// fn compute_bounds(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
//   let particle_index = global_invocation_id.x;
//   let particle = particles[particle_index];
//   let position = particle.position;
//   var bounds_min = vec3<i32>(atomicLoad(&bounds.min_x), atomicLoad(&bounds.min_y), atomicLoad(&bounds.min_z));
//   var bounds_max = vec3<i32>(atomicLoad(&bounds.max_x), atomicLoad(&bounds.max_y), atomicLoad(&bounds.max_z));
//   bounds_min = min(bounds_min, vec3<i32>(vec3<f32>(floor(position.x), floor(position.y), floor(position.z))));
//   bounds_max = max(bounds_max, vec3<i32>(vec3<f32>(ceil(position.x), ceil(position.y), ceil(position.z))));
//   atomicStore(&bounds.min_x, bounds_min.x);
//   atomicStore(&bounds.min_y, bounds_min.y);
//   atomicStore(&bounds.min_z, bounds_min.z);
//   atomicStore(&bounds.max_x, bounds_max.x);
//   atomicStore(&bounds.max_y, bounds_max.y);
//   atomicStore(&bounds.max_z, bounds_max.z);
// }

fn flatten_grid_index(index: vec3<u32>) -> u32 {
  return index.x + index.y * GRID_SIZE + index.z * GRID_SIZE * GRID_SIZE;
}

@compute
@workgroup_size(4, 4, 4)
fn clear_grid(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let grid_position = global_invocation_id;
  let grid_index = flatten_grid_index(grid_position);
  grid[grid_index].particles_length = 0;
}

@compute
@workgroup_size(1)
fn compute_grid(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let particle_index = global_invocation_id.x;
  let particle = particles[particle_index];
  let position = particle.position;
  let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
  let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));
  let grid_position = vec3<u32>(floor((position - bounds_min) / (bounds_max - bounds_min) * f32(GRID_SIZE - 1)));
  let grid_index = flatten_grid_index(grid_position);
  let particles_length = atomicAdd(&grid[grid_index].particles_length, 1);
  grid[grid_index].particles[particle_index] = particle_index;
}

// @compute
// @workgroup_size(64)
// fn compute_octree(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
//   let particle_index = global_invocation_id.x;
//   let particle = particles[particle_index];
//   let node_index = 0;
//   loop {
//     let node = octree_nodes[node_index];
//     let child_node_index = get_child_node_index(particle.position, node.bounds);
//     let child_node = octree_nodes[child_node_index];
//     let particles_length = atomicLoad(&child_node.particles_length);
//     if (particles_length + 1 <= MAX_PARTICLES_PER_NODE) {
//       child_node.particles[particles_length + 1] = particle_index;
//       atomicAdd(&child_node.particles_length, 1);
//     } else {
      
//     }
//   }
// }

// fn get_child_node_index(position: vec3<f32>, bounds: Bounds) -> u32 {
//     let centre = (bounds.min + bounds.max) * 0.5;
//     let octant = vec3<f32>(
//         position.x >= centre.x ? 1.0 : 0.0,
//         position.y >= centre.y ? 1.0 : 0.0,
//         position.z >= centre.z ? 1.0 : 0.0
//     );
//     let index = dot(octant, vec3<f32>(4.0, 2.0, 1.0));
//     return index as u32;
// }
