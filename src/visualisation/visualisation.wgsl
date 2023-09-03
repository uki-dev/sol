const EPSILON = .0001;

const STEP_SIZE = .01;
const MAX_DISTANCE = 32.;
const SHADOW_STEP_SIZE = .01;
const SHADOW_MAX_DISTANCE = 8.;

const LIGHT_COLOUR = vec3<f32>(0.8, 0.8, 0.8);
const LIGHT_DIRECTION = vec3<f32>(.5, 1., -.3);

struct Uniforms {
    // TODO: can we just decompose this from `inverse__view_projection`?
    camera_position: vec3<f32>,
    inverse_view_projection: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

const SPHERE = 0u;
struct Object {
    matrix: mat4x4<f32>,
    colour: vec4<f32>,
    sdf: u32,
}

// TODO: can we instead just store a struct with arrays and use `arrayLength`?
@group(0)
@binding(1)
var<storage, read> objects: array<Object>;

@group(0)
@binding(2)
var<storage, read> objects_length: u32;

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
    let ray_march_result = ray_march(ray_origin, ray_direction);
    if (!ray_march_result.hit) {
        return vec4<f32>(ray_direction, 1.);
    }
    return vec4<f32>(diffuse(ray_march_result.colour.xyz, ray_march_result.normal), ray_march_result.colour.a);
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

fn ray_march(origin: vec3<f32>, direction: vec3<f32>) -> RayMarchResult {
    var result: RayMarchResult;
    var position = origin;
    var distance: f32 = evaluate_scene(position).distance;
    for (var step: f32 = 0.; step < MAX_DISTANCE; step += distance) {
        let evaluate_scene_result = evaluate_scene(position);
        if evaluate_scene_result.distance <= EPSILON {
            result.hit = true;
            result.position = position;
            result.normal = evaluate_scene_normal(position);
            result.colour = evaluate_scene_result.object.colour;
            return result;
        }
        distance = evaluate_scene_result.distance;
        position += direction * distance;
    }
    result.hit = false;
    return result;
}

struct EvaluateSceneResult {
    object: Object,
    distance: f32,
}

fn evaluate_scene(position: vec3<f32>) -> EvaluateSceneResult {
    var result: EvaluateSceneResult; 
    result.distance = MAX_DISTANCE;
    for (var i = 0u; i < objects_length; i += 1u) {
        let object = objects[i];
        // TODO: `object.matrix` be the inverse for world -> local
        let relative_position = (vec4<f32>(position - object.matrix[3].xyz, 1.)).xyz;
        let distance = sphere(relative_position, 0.5);
        if distance < result.distance {
            result.distance = distance;
            result.object = object;
        }
    }
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

// TODO: figure out shader imports

// https://iquilezles.org/articles/distfunctions/

fn sphere(position: vec3<f32>, radius: f32) -> f32 {
    return length(position) - radius;
}

fn sphere_relative(position: vec3<f32>, translation: vec3<f32>, radius: f32) -> f32 {
    return length(position - translation) - radius;
}

fn cube(position: vec3<f32>, extents: vec3<f32>) -> f32 {
    let q = abs(position) - extents;
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