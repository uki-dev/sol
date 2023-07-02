const EPSILON = .0001;

const STEP_SIZE = .01;
const MAX_DISTANCE = 16.;
const SHADOW_STEP_SIZE = .001;
const SHADOW_MAX_DISTANCE = 8.;

const LIGHT_DIRECTION = vec3<f32>(.5, 1., -.3);

struct Uniforms {
    camera_position: vec3<f32>,
    inverse_view_projection: mat4x4<f32>,
}

const AIR = 0u;
const WATER = 1u;
const SAND = 2u;
const SOIL = 3u;

struct Cell {
    material: u32,
    // velocity: vec3<f32>,
}

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

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
    for (var step: f32 = 0.; step < MAX_DISTANCE; step += STEP_SIZE) {
        let position = ray_origin + ray_direction * step;
        let distance = sdf(position);
        if distance <= EPSILON {
            let normal = normal(position);
            let diffuse = max(dot(normal, light_direction), 0.);
            return vec4<f32>(vec3<f32>(1.) * diffuse, 1.0);
        }
    }
    return vec4<f32>(ray_direction, 1.);
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

    // map to grid space and sample cell
    let half_size = vec3<f32>(8., 8., 8.) * .5;
    let grid_position = position + half_size;
    let cell_position = vec3<u32>(floor(grid_position));
    if all(grid_position >= vec3<f32>(0., 0., 0.)) && all(grid_position < vec3<f32>(8., 8., 8.)) {
        // x + y * width + z * width * height
        let cell_index = cell_position.x + cell_position.y * 8u + cell_position.z * 8u * 8u;
        let cell = cell_grid[cell_index];
        sample.cell = cell;
    }

    sample.position = vec3<f32>(cell_position) - half_size - position + 0.5;
    return sample;
}

var<private> neighbours: array<vec3<f32>, 14> = array<vec3<f32>, 14>(
    // left
    vec3<f32>(-1., .0, .0),
    // right
    vec3<f32>(1., .0, .0),
    // bottom
    vec3<f32>(.0, -1., .0),
    // top
    vec3<f32>(.0, 1., .0),
    // back
    vec3<f32>(.0, .0, -1.),
    // front
    vec3<f32>(.0, .0, 1.),
    // back bottom left 
    vec3<f32>(-1., -1., -1.),
    // back bottom right 
    vec3<f32>(1., -1., -1.),
    // back top right 
    vec3<f32>(1., 1., -1.),
    // back top left 
    vec3<f32>(-1., 1., -1.),
    // front bottom left 
    vec3<f32>(-1., -1., 1.),
    // front bottom right 
    vec3<f32>(1., -1., 1.),
    // front top right 
    vec3<f32>(1., 1., 1.),
    // front top left 
    vec3<f32>(-1., 1., 1.)
);

fn sdf(position: vec3<f32>) -> f32 {
    return sharp_union(
        sphere(position + 1., 0.5),
        sphere(position - 1., 0.5),
    );
    // let sample = sample_grid(position);
    // if sample.cell.material != AIR {
    //     var distance = sphere(sample.position, 0.5);
    //     for (var i = 0; i < 14; i++) {
    //         let neighbour = sample_grid(position + neighbours[i]);
    //         if neighbour.cell.material != AIR {
    //             distance = smooth_union(distance, sphere(neighbour.position, 0.5), .2);
    //         }
    //     }
    //     return distance;
    // }
    // return EPSILON * 2.;
}

fn occlusion(position: vec3<f32>, direction: vec3<f32>) -> f32 {
    // return position.y;
    let start: vec3<f32> = position + (normal(position) * .01);
    for (var step: f32 = .01; step < SHADOW_MAX_DISTANCE; step += SHADOW_STEP_SIZE) {
        let distance = sdf(start + (direction * step));
        if distance < EPSILON {
            return .1;
        }
    }
    return 1.;
}

// https://iquilezles.org/articles/distfunctions/

fn sphere(position: vec3<f32>, radius: f32) -> f32 {
    return length(position) - radius;
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
    let a: f32 = sdf(position + vec3(EPSILON, 0., 0.));
    let b: f32 = sdf(position - vec3(EPSILON, 0., 0.));
    let c: f32 = sdf(position + vec3(0., EPSILON, 0.));
    let d: f32 = sdf(position - vec3(0., EPSILON, 0.));
    let e: f32 = sdf(position + vec3(0., 0., EPSILON));
    let f: f32 = sdf(position - vec3(0., 0., EPSILON));

    // return the normalised gradient
    return normalize(vec3<f32>(
        (a - b) / (2. * EPSILON),
        (c - d) / (2. * EPSILON),
        (e - f) / (2. * EPSILON)
    ));
}
