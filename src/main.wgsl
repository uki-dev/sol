alias Position = vec2<f32>;

var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
  vec2<f32>(-1., -1.),
  vec2<f32>( 1., -1.),
  vec2<f32>(-1.,  1.),
  vec2<f32>( 1., -1.),
  vec2<f32>(-1.,  1.),
  vec2<f32>( 1.,  1.),
);

@vertex
fn vertex(
  @builtin(vertex_index) vertex_index: u32,
) -> @builtin(position) vec4<f32> {
  return vec4<f32>(vertices[vertex_index], 0., 1.);
}

fn circle(r: f32, p: Position) -> f32 {
  return length(p) - r;
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  // TODO: Provide uniform or other config for render resolution
  var uv: vec2<f32> = position.xy / vec2<f32>(800., 600.);
  uv -= 0.5;
  var d: f32 = circle(0.25, uv);
  return vec4(d, d, d, 1.);
}