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
var<storage, read_write> buffer: array<Cell>;

const SIZE = 8u;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) position: vec3<u32>) {
    let half_size = f32(SIZE) * 0.5;
    let distance = length(vec3<f32>(position) - vec3<f32>(half_size));
    let index = position.x + position.y * SIZE + position.z * SIZE * SIZE;
    buffer[index].material = select(AIR, SOIL, distance <= half_size);
    // buffer[index].velocity.y = 69.;
}