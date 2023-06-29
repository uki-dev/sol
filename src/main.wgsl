const STEP_SIZE = 0.01;
const MAX_DISTANCE = 12.8;
const SHADOW_STEP_SIZE = 0.01;
const SHADOW_MAX_DISTANCE = 8.;
const EPSILON = 0.001;

const LIGHT_DIRECTION = vec3<f32>(0.5, 1., -0.3);

struct Camera {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_view_projection: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

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
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
) -> Vertex {
    var output: Vertex;
    output.position = vec4<f32>(vertices[vertex_index], 0., 1.);
    output.uv = output.position.xy;
    return output;
}


@fragment
fn fragment(vertex: Vertex) -> @location(0) vec4<f32> {
    var uv: vec2<f32> = vertex.uv;
    var direction = (camera.inverse_view_projection * vec4<f32>(normalize(vec3<f32>(uv, 1.)), 1.)).xyz;
    // TODO: use far plane distance | constant max loop ?
    for (var step: f32 = 0.; step < MAX_DISTANCE; step += STEP_SIZE) {
        var position = direction * step;
        var distance = sdf(position);
        if distance < EPSILON {
            var normal = normal(position);
            var diffuse = max(dot(normal, normalize(LIGHT_DIRECTION)), 0.);
            var shadow = occlusion(position, normalize(LIGHT_DIRECTION));
            return vec4<f32>(1., 1., 1., 1.) * diffuse * shadow;
        }
    }
    return vec4<f32>(direction, 1.);
}

fn occlusion(position: vec3<f32>, direction: vec3<f32>) -> f32 {
  // return position.y;
    var startPosition: vec3<f32> = position + (normal(position) * 0.01);
    for (var step: f32 = 0.; step < SHADOW_MAX_DISTANCE; step += SHADOW_STEP_SIZE) {
        var distance = sdf(startPosition + (direction * step));
        if distance < EPSILON {
            return 0.1;
        }
    }
    return 1.;
}

// https://iquilezles.org/articles/distfunctions/

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

fn sphere(position: vec3<f32>, radius: f32) -> f32 {
    return length(position) - radius;
}

fn box(position: vec3<f32>, dimensions: vec3<f32>) -> f32 {
    var x: f32 = abs(position.x) - dimensions.x;
    var y: f32 = abs(position.y) - dimensions.y;
    var z: f32 = abs(position.z) - dimensions.z;
    return min(min(x, y), z);
}

fn floorPlane(position: vec3<f32>, height: f32) -> f32 {
    return position.y - height;
}

fn sdf(position: vec3<f32>) -> f32 {
    return sharp_union(
        floorPlane(position, -1.),
        smooth_union(
            sphere(position - vec3(0., -.5, 2.), .5),
            smooth_union(
                sphere(position - vec3(-.5, 0., 2.), .5),
                sphere(position - vec3(.5, 0., 2.), .5),
                .5
            ),
            .5,
        )
    );
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
