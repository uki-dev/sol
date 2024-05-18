#import ../common.wgsl as Common

const EPSILON = .0001;

const STEP_SIZE = .01;
const MAX_DISTANCE = 128.;
const SHADOW_STEP_SIZE = .01;
const SHADOW_MAX_DISTANCE = 8.;

const LIGHT_COLOUR = vec3<f32>(0.8, 0.8, 0.8);
const LIGHT_DIRECTION = vec3<f32>(.5, 1., -.3);

@export struct Uniforms {
    // TODO: can we just decompose this from `inverse__view_projection`?
    camera_position: vec3<f32>,
    inverse_view_projection: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var<storage, read> particles: array<Common::Particle>;

@group(0)
@binding(2)
var<storage, read> bounds: Common::Bounds;

@group(0)
@binding(3)
var<storage, read> grid: array<Common::GridCell>;

struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1., -1.),
    vec2<f32>(1., -1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(1., -1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(1., 1.),
);

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
) -> Vertex {
    var output: Vertex;
    output.position = vec4<f32>(vertices[vertex_index], 0., 1.);
    output.ndc = output.position.xy;
    return output;
}

@fragment
fn fragment(vertex: Vertex) -> @location(0) vec4<f32> {
    let ray_origin = uniforms.camera_position;
    let ray_direction = normalize((uniforms.inverse_view_projection * vec4<f32>(vertex.ndc, 1., 1.)).xyz);
    let bounds_min = vec3<f32>(vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z));
    let bounds_max = vec3<f32>(vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z));
    if (ray_box_intersection(ray_origin, ray_direction, bounds_min, bounds_max)) {
        let ray_march_result = ray_march(ray_origin, ray_direction);
        if (ray_march_result.hit) {
            return vec4<f32>(diffuse(ray_march_result.colour.xyz, ray_march_result.normal), ray_march_result.colour.a);
        }
    }
    return vec4<f32>(ray_direction, 1.);
}

fn diffuse(albedo: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let illumination = LIGHT_COLOUR * max(dot(normal, normalize(LIGHT_DIRECTION)), 0.);
    return albedo * illumination;
}

struct RayMarchResult {
    hit: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    colour: vec4<f32>,
}

fn ray_box_intersection(origin: vec3<f32>, direction: vec3<f32>, box_min: vec3<f32>, box_max: vec3<f32>) -> bool {
    let t_min = (box_min - origin) / direction;
    let t_max = (box_max - origin) / direction;
    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);
    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);
    return t_far >= t_near;
}

fn ray_march_adaptive(origin: vec3<f32>, direction: vec3<f32>) -> RayMarchResult {
    var result: RayMarchResult;
    var position = origin;
    var distance: f32 = evaluate_scene(position).distance;
    for (var step: f32 = 0.; step < MAX_DISTANCE; step += distance) {
        let evaluate_scene_result = evaluate_scene(position);
        if evaluate_scene_result.distance <= EPSILON {
            result.hit = true;
            result.position = position;
            result.normal = evaluate_scene_normal(position);
            // result.colour = evaluate_scene_result.object.colour;
            result.colour = vec4<f32>(1.0);
            return result;
        }
        distance = evaluate_scene_result.distance;
        position += direction * distance;
    }
    result.hit = false;
    return result;
}

fn ray_march(origin: vec3<f32>, direction: vec3<f32>) -> RayMarchResult {
    var result: RayMarchResult;
    for (var step: f32 = 0.; step < MAX_DISTANCE; step += STEP_SIZE) {
        let position = origin + direction * step;
        let evaluate_scene_result = evaluate_scene(position);
        if evaluate_scene_result.distance <= EPSILON {
            result.hit = true;
            result.position = position;
            result.normal = evaluate_scene_normal(position);
            // result.colour = evaluate_scene_result.object.colour;
            result.colour = vec4<f32>(1.0);
            return result;
        }
    }
    result.hit = false;
    return result;
}

struct EvaluateSceneResult {
    // object: Object,
    distance: f32,
}

fn evaluate_scene(position: vec3<f32>) -> EvaluateSceneResult {
    var result: EvaluateSceneResult; 
    result.distance = MAX_DISTANCE;
    // result.distance = sphere(position - vec3<f32>(0.0), 0.5);
    result = evaluate_grid(position);
  
    return result;
}

fn evaluate_scene_normal(position: vec3<f32>) -> vec3<f32> {
    // numerical gradient estimation
    let a: f32 = evaluate_scene(position + vec3(EPSILON, 0., 0.)).distance;
    let b: f32 = evaluate_scene(position - vec3(EPSILON, 0., 0.)).distance;
    let c: f32 = evaluate_scene(position + vec3(0., EPSILON, 0.)).distance;
    let d: f32 = evaluate_scene(position - vec3(0., EPSILON, 0.)).distance;
    let e: f32 = evaluate_scene(position + vec3(0., 0., EPSILON)).distance;
    let f: f32 = evaluate_scene(position - vec3(0., 0., EPSILON)).distance;
    return normalize(vec3<f32>(
        (a - b),
        (c - d),
        (e - f)
    ));
}

fn evaluate_grid(position: vec3<f32>) -> EvaluateSceneResult{
    var result: EvaluateSceneResult; 
    let bounds_min = vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z);
    let bounds_max = vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z);
    let grid_position = Common::world_position_to_grid_position(position, bounds);
    let bounded_grid_position = clamp(grid_position, vec3<i32>(bounds_min), vec3<i32>(bounds_max));
    let grid_index = Common::grid_position_to_grid_index(bounded_grid_position);
    let particles_length = grid[grid_index].particles_length;
    if (particles_length == 0) {
        result.distance = MAX_DISTANCE;
        return result;
    }
    result.distance = evaluate_particle(grid_index, 0u, position);
    for (var i = 1u; i < particles_length; i++) {
        result.distance = smooth_union(result.distance, evaluate_particle(grid_index, i, position), 1.0);
    }
    return result;
}

fn evaluate_particle(grid_index: i32, cell_particle_index: u32, position: vec3<f32>) -> f32 {
    let particle_index = grid[grid_index].particles[cell_particle_index];
    let particle = particles[particle_index];
    let relative_position = position - particle.position;
    return sphere(relative_position, 0.5);
}

// fn occlusion(position: vec3<f32>, direction: vec3<f32>) -> f32 {
//     // return position.y;
//     let start: vec3<f32> = position + (normal(position) * .01);
//     for (var step: f32 = .01; step < SHADOW_MAX_DISTANCE; step += SHADOW_STEP_SIZE) {
//         let result = map(start + (direction * step));
//         if result.distance < EPSILON {
//             return .1;
//         }
//     }
//     return 1.;
// }

// https://iquilezles.org/articles/distfunctions/

fn sphere(position: vec3<f32>, radius: f32) -> f32 {
    return length(position) - radius;
}

fn sphere_relative(position: vec3<f32>, translation: vec3<f32>, radius: f32) -> f32 {
    return length(position - translation) - radius;
}

fn cube(position: vec3<f32>, extent: vec3<f32>) -> f32 {
    let q = abs(position) - extent;
    return length(max(q, vec3<f32>(.0))) + min(max(q.x, max(q.y, q.z)), .0);
}

fn plane_y_infinite(position: vec3<f32>) -> f32 {
    return position.y;
}

fn sharp_union(a: f32, b: f32) -> f32 { return min(a, b); }

fn sharp_subtraction(a: f32, b: f32) -> f32 { return max(-a, b); }

fn sharp_intersection(a: f32, b: f32) -> f32 { return max(a, b); }

fn smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(.5 + .5 * (b - a) / k, .0, 1.);
    return mix(b, a, h) - k * h * (1. - h);
}

fn smooth_subtraction(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(.5 - .5 * (b + a) / k, .0, 1.);
    return mix(b, -a, h) + k * h * (1. - h);
}

fn smooth_intersection(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(.5 - .5 * (b - a) / k, .0, 1.);
    return mix(b, a, h) + k * h * (1. - h);
}