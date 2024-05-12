struct Particle {
  position: vec3<f32>,
}

struct Bounds {
  min_x: atomic<i32>,
  min_y: atomic<i32>,
  min_z: atomic<i32>,
  max_x: atomic<i32>,
  max_y: atomic<i32>,
  max_z: atomic<i32>,
}

// struct OctreeNode {
//   bounds: Bounds,
//   children: array<u32, 8>,
//   particles: array<u32>,
//   particles_length: atomic<u32>,
// }

@group(0)
@binding(0)
var<storage, read> particles: array<Particle>;

@group(0)
@binding(1)
var<storage, read_write> bounds: Bounds;

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

// @compute
// @workgroup_size(64)
// fn partition(@builtin(global_invocation_id) position: vec3<u32>) {
//   let particle_index = global_id.x as u32;
//   let particle = particles[particle_index];
//   loop {
//     let child_index = get_child_node_index(particle.position)
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
