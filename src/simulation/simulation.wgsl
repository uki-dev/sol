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

const ITERATIONS = 1u;

const GRAVITY = vec3<f32>(0.0, -9.81, 0.0);

@compute
@workgroup_size(1)
fn simulate(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    // TODO: Use per-particle or per-material properties
    let mass = 1.0;
    let frictional_coefficient = 0.5;

    let particle_index = global_invocation_id.x;
    var previous_position = particles[particle_index].old_position;
    var current_position = particles[particle_index].position;

    let delta_time = uniforms.delta_time / f32(ITERATIONS);
    let delta_time_squared = delta_time * delta_time;

    let velocity = (current_position - previous_position) / delta_time;
    let gravitational_force = GRAVITY * mass;
    let frictional_force = -velocity * frictional_coefficient;
    let acceleration = (gravitational_force + frictional_force) / mass;

    let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
    let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));
    for (var i = 0u; i < ITERATIONS; i++) {
        // Solve inter-particle collision
        for (var i = 1u; i < Common::MAX_PARTICLES; i++) {
            if (i == particle_index) {
                continue; // Skip self-collision
            }
            current_position = solve_collision(
                current_position,
                particles[i].position,
                // TODO: Replace this with actual particle radius
                PARTICLE_RADIUS + PARTICLE_RADIUS,
            );
        }

        // Apply verlet integration
        let velocity = (current_position - previous_position); // * damping;
        let next_position = current_position + velocity + acceleration * delta_time_squared;
        previous_position = current_position;
        current_position = next_position;

        // Solve bounding box collision
        if (current_position.x < bounds_min.x) {
            current_position.x = bounds_min.x;
        } else if (current_position.x > bounds_max.x) {
            current_position.x = bounds_max.x;
        }
        if (current_position.y < bounds_min.y) {
            current_position.y = bounds_min.y;
        } else if (current_position.y > bounds_max.y) {
            current_position.y = bounds_max.y;
        }
        if (current_position.z < bounds_min.z) {
            current_position.z = bounds_min.z;
        } else if (current_position.z > bounds_max.z) {
            current_position.z = bounds_max.z;
        }
    }

    particles[particle_index].old_position = previous_position;
    particles[particle_index].position = current_position;
}

// fn integrate(particle_index: u32, position: vec3<v32>) {

// }

fn process_particle_collisions(particle_index: u32, position: vec3<f32>) -> vec3<f32> {
    var processed_position = position;
    let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
    let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));

    // Process inter-particle collision with neighbouring particles
    // let grid_index = Common::world_position_to_grid_index(processed_position, bounds);
    // let particles_length = grid[grid_index].particles_length;
    // for (var i = 1u; i < particles_length; i++) {
    //     if (i == particle_index){
    //         continue; // Skip self-collision
    //     } 

    //     var neighbouring_particle_position = particles[i].position;
    //     processed_position = solve_collision(
    //         processed_position,
    //         neighbouring_particle_position,
    //         // TODO: Replace this with actual particle radius
    //         PARTICLE_RADIUS + PARTICLE_RADIUS,
    //     );
    // }
    return processed_position;
}

// fn solve_verlet_integration(current_position: vec3<f32>, previous_position: vec3<f32>, delta_time_squared: f32) -> vec3<f32> {
//     let velocity = (current_position - previous_position);
//     return current_position + velocity + acceleration * delta_time_squared;
// }

fn solve_collision(
    position: vec3<f32>,
    collision_position: vec3<f32>,
    radius: f32,
) -> vec3<f32> {
    // TODO: Use per-particle or per-material restitution factor?
    let restitution = 0.0; // Coefficient of restitution (bounciness)
    var adjusted_position = position;
    let direction = collision_position - position;
    let distance = length(direction);
    let min_distance = radius;
    if (distance < min_distance) {
        let normal = normalize(direction);
        let penetration = (min_distance - distance) * normal;
        adjusted_position -= (penetration * 1.0 + restitution) * 0.5;
    }
    return adjusted_position;
}