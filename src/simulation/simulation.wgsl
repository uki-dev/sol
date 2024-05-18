#import ../common.wgsl as Common

@export struct Uniforms {
    delta_time: f32,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var<storage, read_write> particles: array<Common::Particle>;

@group(0)
@binding(2)
var<storage, read> bounds: Common::Bounds;

@group(0)
@binding(3)
var<storage, read> grid: array<Common::GridCell>;

// TODO: Replace this with actual particle radius
const PARTICLE_RADIUS = 0.5;

const PHYSICS_ITERATIONS = 64u;

@compute
@workgroup_size(1)
fn simulate(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let mass = 100.0;
    // let restitution = 0.8; // Coefficient of restitution (bounciness)
    // let damping = 0.99; // Damping factor for friction (0.99 for slight friction)
    let particle_index = global_invocation_id.x;
    let position = particles[particle_index].position;
    var acceleration = vec3<f32>();
    acceleration += vec3<f32>(0.0, -9.81, 0.0) * mass; // Gravity
    acceleration /= mass;

    var current_position = particles[particle_index].position;
    var previous_position = particles[particle_index].old_position;
    var new_position = current_position;

    let delta_time = uniforms.delta_time / f32(PHYSICS_ITERATIONS);
    let delta_time_squared = delta_time * delta_time;
    for (var i = 0u; i < PHYSICS_ITERATIONS; i++) {
        let velocity = (current_position - previous_position); // * damping;
        velocity = process_particle_collisions(particle_index, position, velocity);
        new_position = current_position + velocity + acceleration * delta_time_squared;
        previous_position = current_position;
        current_position = new_position;
    }
    // new_position = process_particle_collisions(particle_index, new_position);

    // Update positions
    particles[particle_index].old_position = current_position;
    particles[particle_index].position = new_position;
}

// fn integrate(particle_index: u32, position: vec3<v32>) {

// }

fn process_particle_collisions(particle_index: u32, position: vec3<f32>) -> vec3<f32> {
    var processed_position = position;
    let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
    let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));
    // if (position.x < bounds_min.x) {
    //     PROCE
    //     acceleration.x = -acceleration.x * restitution;
    // } else if (position.x > bounds_max.x) {A

    // }
    // if (position.y < bounds_min.y || position.y > bounds_max.y) {
    //     acceleration.y = -acceleration.y * restitution;
    // }
    // if (position.z < bounds_min.z || position.z > bounds_max.z) {
    //     acceleration.z = -acceleration.z * restitution;
    // }

    // Process inter-particle collision with neighbouring particles
    // let grid_index = Common::world_position_to_grid_index(processed_position, bounds);
    // let particles_length = grid[grid_index].particles_length;
    // for (var i = 1u; i < particles_length; i++) {
    //     if (i == particle_index){
    //         continue; // Skip self-collision
    //     } 

    //     var neighbouring_particle_position = particles[i].position;
    //     processed_position = process_collision(
    //         processed_position,
    //         neighbouring_particle_position,
    //         // TODO: Replace this with actual particle radius
    //         PARTICLE_RADIUS + PARTICLE_RADIUS,
    //     );
    // }
    return processed_position;
}

fn process_boundary_collision(
    position: vec3<f32>,
    radius: f32,
    bounds: Common::Bounds,
) -> vec3<f32> {
    var processed_position = position;
    let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
    let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));
    for (var axis = 0u; axis < 3u; axis++) {
        var collision_position = position;
        collision_position[axis] = bounds_min[axis];
        processed_position = process_collision(
            processed_position,
            collision_position,
            radius
        );
        collision_position[axis] = bounds_max[axis];
        processed_position = process_collision(
            processed_position,
            collision_position,
            radius
        );
    }
    return processed_position;
}

fn process_collision(
    position: vec3<f32>,
    collision_position: vec3<f32>,
    radius: f32,
    // restitution: f32
) -> vec3<f32> {
    var adjusted_position = position;
    let direction = collision_position - position;
    let distance = length(direction);
    let min_distance = radius * 2.0;
    if (distance < min_distance) {
        // Resolve collision
        let normal = normalize(direction);
        let penetration = (min_distance - distance) * normal;
        adjusted_position += penetration; // Move particle out of collision
        // adjusted_position -= normal * (1.0 + restitution);
    }
    return adjusted_position;
}