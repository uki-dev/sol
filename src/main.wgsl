// TODO: move to compute shader ? might still need vertex + fragment for some systems however

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

// TODO: move to sdf include
fn circle(r: f32, p: vec2<f32>) -> f32 {
    return length(p) - r;
}

// TODO: move to sdf include
fn sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

const STEP_SIZE = 0.01;
const MAX_STEPS = 128.;
const EPSILON = 0.001;

@fragment
fn fragment(vertex: Vertex) -> @location(0) vec4<f32> {
    var uv: vec2<f32> = vertex.uv.xy;
    // TODO: aspect ratio correction (this shouldn't be needed when we use a projection matrix though)
    var direction = normalize(vec3(uv, 1.0));
    // TODO: use far plane distance | constant max loop ?
    for (var step: f32 = 0.; step < MAX_STEPS; step += STEP_SIZE) {
        var position = direction * step;
        var distance = sphere(position - vec3(0., 0., 2.0), 0.5);
        if distance < EPSILON {
            return vec4(1., 1., 1., 1.);
        }
    }
    return vec4(direction, 1.);
}