#import ../common.wgsl as Common

struct AtomicVec3I32 {
  x: atomic<i32>,
  y: atomic<i32>,
  z: atomic<i32>,
}

struct AtomicBounds {
  min: AtomicVec3I32,
  max: AtomicVec3I32,
}

@group(0)
@binding(0)
var<storage, read> particles: array<Common::Particle>;


@group(0)
@binding(1)
var<storage, read_write> bounds: AtomicBounds;

@compute
@workgroup_size(256)
fn calculate_bounds(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let particle_index = global_invocation_id.x;
  let particle = particles[particle_index];
  atomicMin(&bounds.min.x, i32(floor(particle.position.x)));
  atomicMin(&bounds.min.y, i32(floor(particle.position.y)));
  atomicMin(&bounds.min.z, i32(floor(particle.position.z)));
  // atomicStore(&bounds.min.z, 200);
  atomicMax(&bounds.max.x, i32(ceil(particle.position.x)));
  atomicMax(&bounds.max.y, i32(ceil(particle.position.y)));
  atomicMax(&bounds.max.z, i32(ceil(particle.position.z)));
}
