#import ../common.wgsl as Common

const PI = 3.141592653589793;

const EPSILON = .001;

const STEP_SIZE = .5;
const MAX_DISTANCE = 128.;
const SHADOW_STEP_SIZE = .01;
const SHADOW_MAX_DISTANCE = 8.;

const LIGHT_COLOUR = vec3<f32>(.8, .8, .8);
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
        let ray_march_result = ray_march_adaptive(ray_origin, ray_direction);
        if (ray_march_result.hit) {
            // TODO: Calculate this in the actual functions that return SDF so that we can use different SDF mapping where desired
            // let uv = sphere_uv(ray_march_result.position);
            let colour = ray_march_result.colour;
            let d = diffuse(colour, ray_march_result.normal);
            let m = metallic(vec3<f32>(1.), ray_march_result.normal, ray_direction);
            let f = vec3<f32>(fresnel(ray_march_result.normal, ray_direction, 1.5));

            return vec4<f32>(m * f + (d * (vec3<f32>(1.) - f)), 1.);
        }
    }
    return vec4<f32>(background(ray_direction), 1.);
}

fn diffuse(albedo: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let i0 = background(normal);
    let i1 = background(spread_rays(normal, 45.0, 0u, 4u));
    let i2 = background(spread_rays(normal, 45.0, 1u, 4u));
    let i3 = background(spread_rays(normal, 45.0, 2u, 4u));
    let i4 = background(spread_rays(normal, 45.0, 3u, 4u));
    let illumination = (i0 + i1 + i2 + i3 + i4) / vec3<f32>(5.0);
    return albedo * illumination;
}


fn spread_rays(original_ray: vec3<f32>, spread_degrees: f32, ray_index: u32, num_rays: u32) -> vec3<f32> {
    let half_spread_rad = radians(spread_degrees * 0.5);
    let angle_step_rad = radians(spread_degrees) / (f32(ray_index) * 2.0 + 1.0);
    let angle_offset_rad = f32(ray_index - num_rays / 2) * angle_step_rad;

    let rotation_axis = cross(original_ray, vec3<f32>(0.0, 0.0, 1.0));
    let rotation_matrix = mat3x3<f32>(
        cos(angle_offset_rad) + rotation_axis.x * rotation_axis.x * (1.0 - cos(angle_offset_rad)),
        rotation_axis.x * rotation_axis.y * (1.0 - cos(angle_offset_rad)) - rotation_axis.z * sin(angle_offset_rad),
        rotation_axis.x * rotation_axis.z * (1.0 - cos(angle_offset_rad)) + rotation_axis.y * sin(angle_offset_rad),

        rotation_axis.x * rotation_axis.y * (1.0 - cos(angle_offset_rad)) + rotation_axis.z * sin(angle_offset_rad),
        cos(angle_offset_rad) + rotation_axis.y * rotation_axis.y * (1.0 - cos(angle_offset_rad)),
        rotation_axis.y * rotation_axis.z * (1.0 - cos(angle_offset_rad)) - rotation_axis.x * sin(angle_offset_rad),

        rotation_axis.x * rotation_axis.z * (1.0 - cos(angle_offset_rad)) - rotation_axis.y * sin(angle_offset_rad),
        rotation_axis.y * rotation_axis.z * (1.0 - cos(angle_offset_rad)) + rotation_axis.x * sin(angle_offset_rad),
        cos(angle_offset_rad) + rotation_axis.z * rotation_axis.z * (1.0 - cos(angle_offset_rad))
    );

    return normalize(rotation_matrix * original_ray);
}


fn fresnel(normal: vec3<f32>, incident_direction: vec3<f32>, ior_ratio: f32) -> f32 {
    let similarity = dot(normal, vec3<f32>(-1) * incident_direction);
    return pow(1. - similarity, 5.0);
}


fn metallic(colour: vec3<f32>, normal: vec3<f32>, incoming: vec3<f32>) -> vec3<f32> {
    let illumination = background(reflect(incoming, normal));
    return colour * illumination;
}

fn reflect(direction: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let dotProduct = dot(direction, normal);
    return direction - 2.0 * dotProduct * normal;
}


fn background(normal: vec3<f32>) -> vec3<f32> {
    // return vec3<f32>(normal.z);
    return normal;
}

struct RayMarchResult {
    hit: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
    colour: vec3<f32>,
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
            result.colour = vec3<f32>(1., 0.5, 0.3);
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
            result.colour = vec3<f32>(1.);
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
    result = evaluate_particles(position);
  
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

fn evaluate_particles(position: vec3<f32>) -> EvaluateSceneResult {
    var result: EvaluateSceneResult; 
    result.distance = evaluate_particle(position, 0u);
    for (var i = 1u; i < Common::MAX_PARTICLES; i++) {
        result.distance = smooth_union(result.distance, evaluate_particle(position, i), 3.);
    }
    return result;
}

fn evaluate_particle(position: vec3<f32>, particle_index: u32) -> f32 {
    let particle = particles[particle_index];
    let relative_position = position - particle.position;
    return sphere(relative_position, Common::PARTICLE_RADIUS);
}

fn evaluate_grid(position: vec3<f32>) -> EvaluateSceneResult {
    var result: EvaluateSceneResult; 
    let bounds_min = vec3<i32>(bounds.min_x, bounds.min_y, bounds.min_z);
    let bounds_max = vec3<i32>(bounds.max_x, bounds.max_y, bounds.max_z);
    let grid_position = Common::world_position_to_grid_position(position, bounds);
    let bounded_grid_position = clamp(grid_position, vec3<i32>(bounds_min), vec3<i32>(bounds_max));
    let grid_index = Common::grid_position_to_grid_index(bounded_grid_position);
    let particles_length = grid[grid_index].particles_length;
    result.distance = MAX_DISTANCE;
    for (var i = 0u; i < particles_length; i++) {
        result.distance = smooth_union(result.distance, evaluate_cell_particle(position, grid_index, i), 3.);
    }
    return result;
}

fn evaluate_cell_particle(position: vec3<f32>, grid_index: i32, cell_particle_index: u32) -> f32 {
    let particle_index = grid[grid_index].particles[cell_particle_index];
    return evaluate_particle(position, particle_index);
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

fn sand_texture(uv: vec2<f32>) -> vec3<f32> {
    let n1 = noise(uv * 512.0);
    let n2 = noise(uv * 1024.0);
    let base_sand_colour = vec3<f32>(0.76, 0.70, 0.50);
    let dark_sand_colour = base_sand_colour * -1.0;
    let light_sand_colour = base_sand_colour * 2.0;
    let colour = mix(mix(base_sand_colour, dark_sand_colour, n1), light_sand_colour, 0.8 + n2 * 0.2);
    return colour;
}

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    let u = f * f * (3.0 - 2.0 * f);

    return mix(a, b, u.x) +
           (c - a) * u.y * (1.0 - u.x) +
           (d - b) * u.x * u.y;
}

fn sphere_uv(position: vec3<f32>) -> vec2<f32> {
    let normal = normalize(position);
    let theta = atan2(normal.y, normal.x);
    let phi = acos(normal.z);
    let u = (theta + PI) / (2.0 * PI);
    let v = phi / PI;
    return vec2<f32>(u, v);
}

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