#import ../common.wgsl as Common

@group(0)
@binding(0)
var<storage, read_write> particles: array<Common::Particle>;

@group(0)
@binding(1)
var<storage, read> bounds: Common::Bounds;

@group(0)
@binding(2)
var<storage, read> grid: array<Common::GridCell>;

// @group(0)
// @binding(3)
// var<uniform, read> grid: array<Uniforms>;

// TODO: Replace this with actual particle radius
const PARTICLE_RADIUS = 0.5;

@compute
@workgroup_size(1)
fn simulate(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let particle_index = global_invocation_id.x;
    let position = particles[particle_index].position;
    var acceleration = vec3<f32>();
    // // Gravity
    acceleration += vec3<f32>(0.0, -9.81, 0.0) * 0.001;

    let restitution = 0.8; // coefficient of restitution (bounciness)
    let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
    let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));
    if (position.x < bounds_min.x || position.x > bounds_max.x) {
        acceleration.x = -acceleration.x * restitution;
    }
    if (position.y < bounds_min.y || position.y > bounds_max.y) {
        acceleration.y = -acceleration.y * restitution;
    }
    if (position.z < bounds_min.z || position.z > bounds_max.z) {
        acceleration.z = -acceleration.z * restitution;
    }

    // Apply mass 
    // let mass = 1.0;
    // acceleration /= mass;


    // Damping factor for friction (0.99 for slight friction)
    let damping = 0.99;

    // Verlet integration with friction
    var current_position = particles[particle_index].position;
    let velocity = (current_position - particles[particle_index].old_position) * damping;
    var new_position = current_position + velocity + acceleration;

    // Process inter-particle collision with neighbouring particles
    let grid_index = Common::world_position_to_grid_index(position, bounds);
    let particles_length = grid[grid_index].particles_length;
    for (var i = 1u; i < particles_length; i++) {
      if (i == particle_index){
        continue; // Skip self-collision
      } 

      var neighbouring_particle_position = particles[i].position;
    //   new_position = calculate_collision_response(
    //     particle.position,
    //     new_position,
    //     particles[i].position,
    //     particle.radius,
    //     restitution
    // );
    }

    // Update positions
    particles[particle_index].old_position = current_position;
    particles[particle_index].position = new_position;
}

fn calculate_collision_response(
    position: vec3<f32>,
    collision_position: vec3<f32>,
    radius: f32,
    // restitution: f32
) -> vec3<f32> {
    let adjusted_position = position;
    let direction = collision_position - position;
    let distance = length(direction);
    let min_distance = radius * 2.0;
    if (distance < min_distance) {
        // Resolve collision
        let normal = normalize(direction);
        let penetration = (min_distance - distance) * normal;
        let adjusted_position = position + penetration; // Move particle out of collision
        // adjusted_position -= normal * (1.0 + restitution);
    }
    return adjusted_position;
}