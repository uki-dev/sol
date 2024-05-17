#import ../common.wgsl as Common

@group(0)
@binding(0)
var<storage, read> particles: array<Common::Particle>;


@group(0)
@binding(1)
var<storage, read_write> bounds: Common::Bounds;

@compute
@workgroup_size(1)
fn calculate_bounds(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
  let particle_index = global_invocation_id.x;
  let particle = particles[particle_index];
  atomicMin(&bounds.min_x, i32(floor(particle.position.x)));
  atomicMin(&bounds.min_y, i32(floor(particle.position.y)));
  atomicMin(&bounds.min_z, i32(floor(particle.position.z)));
  atomicMax(&bounds.max_x, i32(ceil(particle.position.x)));
  atomicMax(&bounds.max_y, i32(ceil(particle.position.y)));
  atomicMax(&bounds.max_z, i32(ceil(particle.position.z)));
}
