const EPSILON = .0001;

const STEP_SIZE = .001;
const MAX_DISTANCE = 256.;
const SHADOW_STEP_SIZE = .01;
const SHADOW_MAX_DISTANCE = 8.;

const LIGHT_DIRECTION = vec3<f32>(.5, 1., -.3);

struct Uniforms {
    width: u32,
    height: u32,
    depth: u32,
    camera_position: vec3<f32>,
    inverse_view_projection: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

const AIR = 0u;
const WATER = 1u;
const SAND = 2u;
const SOIL = 3u;

struct Cell {
    material: u32,
    // velocity: vec3<f32>,
}

@group(0)
@binding(1)
var<storage, read> cell_grid: array<Cell>;

var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1., -1.),
    vec2<f32>(1., -1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(1., -1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(1., 1.),
);

struct Vertex {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

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
    var ndc = vertex.ndc;
    let ray_direction = normalize((uniforms.inverse_view_projection * vec4<f32>(ndc, 1., 1.)).xyz);
    let ray_origin = uniforms.camera_position;
    let light_direction = normalize(LIGHT_DIRECTION);
    var position = ray_origin;
    var distance: f32 = map(position).distance;
    var hit: SurfaceHit;
    hit.material = AIR;
    hit.position = ray_origin + (ray_direction * MAX_DISTANCE);
    for (var step: f32 = 0.; step < MAX_DISTANCE; step += distance) {
        position += ray_direction * distance;
        let result: SDFResult = map(position);
        distance = result.distance;
        if distance <= EPSILON {
            let normal = normal(position);
            hit.normal = normal;
            hit.position = position;
            hit.material = result.nearest_surface_material;
            break;
        }
    }

    let distance_from_cam = distance(ray_origin, hit.position);
    let distance_deriv = fwidth(distance_from_cam);
    // Shading
    if (hit.material == AIR) {
        // Only the material property of hit is defined if no hit occurs
        return vec4<f32>(ray_direction, 1.);
    } else if (hit.material == SAND) {
        if (distance_deriv > 2.0) {
            return vec4<f32>(0., 0., 0., 1.);
        }
        let sand_color = vec3<f32>(0.9, 0.7, 0.3);
        let illumination = max(dot(hit.normal, light_direction), 0.);
        return vec4<f32>(diffuse_illumination(sand_color, vec3<f32>(illumination)), 1.);
    } else {
        // Should not reach here once all materials are defined
        return vec4<f32>(0.);
    }
}

fn diffuse_illumination(albedo: vec3<f32>, illumination: vec3<f32>) -> vec3<f32> {
    return albedo * illumination;
}

struct SurfaceHit {
    normal: vec3<f32>,
    position: vec3<f32>,
    material: u32,
}

struct GridSample {
    cell: Cell,
    position: vec3<f32>,
}

fn sample_grid(position: vec3<f32>) -> GridSample {
    // setup default sample
    var sample: GridSample;
    var empty_cell: Cell;
    empty_cell.material = AIR;
    sample.cell = empty_cell;

    // map to grid space and sample cellE
    let dimensions = vec3<u32>(uniforms.width, uniforms.height, uniforms.depth);
    let extents = vec3<f32>(dimensions) * .5;
    let grid_position = position + extents;
    let cell_position = vec3<u32>(floor(grid_position));
    if all(grid_position >= vec3<f32>(0., 0., 0.)) && all(grid_position < vec3<f32>(dimensions)) {
        // x + y * width + z * width * height
        let cell_index = cell_position.x + cell_position.y * uniforms.width + cell_position.z * uniforms.width * uniforms.height;
        let cell = cell_grid[cell_index];
        sample.cell = cell;
    }

    sample.position = vec3<f32>(cell_position) - extents + 0.5;
    return sample;
}

fn grid_sdf(position: vec3<f32>) -> f32 {
    let extents = 4.;
    var distance = 0.5; //TODO: Replace with grid cell size
    let sample = sample_grid(position);
    if sample.cell.material != AIR {
        for (var x = -extents; x <= extents; x += 1.) {
            for (var y = -extents; y <= extents; y += 1.) {
                for (var z = -extents; z <= extents; z += 1.) {
                    let offset = vec3<f32>(x, y, z);
                    let sample = sample_grid(position + offset);
                    if sample.cell.material != AIR {
                        distance = smooth_union(distance, sphere_relative(position, sample.position, 2.), 32.);
                    }
                }
            }
        }
    }
    return distance;
}

struct SDFResult {
    distance: f32,
    nearest_surface_material: u32,
}

fn map(position: vec3<f32>) -> SDFResult {
    let distance = grid_sdf(position);
    var result: SDFResult;
    result.distance = distance;
    result.nearest_surface_material = SAND;
    return result;

}

fn occlusion(position: vec3<f32>, direction: vec3<f32>) -> f32 {
    // return position.y;
    let start: vec3<f32> = position + (normal(position) * .01);
    for (var step: f32 = .01; step < SHADOW_MAX_DISTANCE; step += SHADOW_STEP_SIZE) {
        let result = map(start + (direction * step));
        if result.distance < EPSILON {
            return .1;
        }
    }
    return 1.;
}

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

// numerical gradient estimation
fn normal(position: vec3<f32>) -> vec3<f32> {
    let a: f32 = map(position + vec3(EPSILON, 0., 0.)).distance;
    let b: f32 = map(position - vec3(EPSILON, 0., 0.)).distance;
    let c: f32 = map(position + vec3(0., EPSILON, 0.)).distance;
    let d: f32 = map(position - vec3(0., EPSILON, 0.)).distance;
    let e: f32 = map(position + vec3(0., 0., EPSILON)).distance;
    let f: f32 = map(position - vec3(0., 0., EPSILON)).distance;

    // return the normalised gradient
    return normalize(vec3<f32>(
        (a - b) / (2. * EPSILON),
        (c - d) / (2. * EPSILON),
        (e - f) / (2. * EPSILON)
    ));
}