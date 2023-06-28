const STEP_SIZE = 0.01;
const MAX_STEPS = 128.;
const EPSILON = 0.001;

const LIGHT_DIRECTION = vec3<f32>(-0.25, -0.75, 0.5);

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
    for (var step: f32 = 0.; step < MAX_STEPS; step += STEP_SIZE) {
        var position = direction * step;
        var distance = sdf(position);
        if distance < EPSILON {
            var normal = normal(position);
            var diffuse = max(dot(normal, normalize(-LIGHT_DIRECTION)), 0.);
            return vec4<f32>(1., 1., 1., 1.) * diffuse;
        }
    }
    return vec4<f32>(direction, 1.);
}

fn sdf_union(a: f32, b: f32) -> f32 { return min(a, b); }

fn sdf_subtraction(a: f32, b: f32) -> f32 { return max(-a, b); }

fn sdf_intersection(a: f32, b: f32) -> f32 { return max(a, b); }

fn sdf_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}

fn sdf_smooth_subtraction(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 + d1) / k, 0.0, 1.0);
    return mix(d2, -d1, h) + k * h * (1.0 - h);
}

fn sdf_smooth_intersection(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 - 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) + k * h * (1.0 - h);
}

fn sphere(position: vec3<f32>, radius: f32) -> f32 {
    return length(position) - radius;
}

// entire sdf composition
fn sdf(position: vec3<f32>) -> f32 {
    return sdf_union(
        sphere(position - vec3(0., 0., 2.), 0.5),
        sdf_union(
            sphere(position - vec3(-2., 0., 2.), 0.5),
            sphere(position - vec3(2., 0., 2.), 0.5)
        )
    );
}

// numerical gradient estimation
fn normal(position: vec3<f32>) -> vec3<f32> {
    let d1: f32 = sdf(position + vec3(EPSILON, 0., 0.));
    let d2: f32 = sdf(position - vec3(EPSILON, 0., 0.));
    let d3: f32 = sdf(position + vec3(0., EPSILON, 0.));
    let d4: f32 = sdf(position - vec3(0., EPSILON, 0.));
    let d5: f32 = sdf(position + vec3(0., 0., EPSILON));
    let d6: f32 = sdf(position - vec3(0., 0., EPSILON));

    // return the normalised gradient
    return normalize(vec3<f32>(
        (d1 - d2) / (2. * EPSILON),
        (d3 - d4) / (2. * EPSILON),
        (d5 - d6) / (2. * EPSILON)
    ));
}
