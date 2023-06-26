var<private> vertices: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
  vec2<f32>(-1., -1.),
  vec2<f32>(1., -1.),
  vec2<f32>(0., 1.),
);

@vertex
fn vertex(
  @builtin(vertex_index) vertex_index: u32,
) -> @builtin(position) vec4<f32> {
  return vec4<f32>(vertices[vertex_index], 0., 1.);
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  return vec4(1., 1., 1., 1.);
}