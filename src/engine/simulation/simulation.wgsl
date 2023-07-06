const AIR = 0u;
const WATER = 1u;
const SAND = 2u;
const SOIL = 3u;

struct Cell {
    material: u32,
    // velocity: vec3<f32>,
}

struct Uniforms {
    width: u32,
    height: u32,
    depth: u32,
}


@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var<storage, read_write> buffer: array<Cell>;

@compute
@workgroup_size(1)
fn populate(@builtin(global_invocation_id) position: vec3<u32>) {
    let extents = vec3<f32>(vec3<u32>(uniforms.width, uniforms.height, uniforms.depth)) * .5;
    let distance = length(vec3<f32>(position) - extents + 0.5);
    let index = position.x + position.y * uniforms.width + position.z * uniforms.width * uniforms.height;
    buffer[index].material = select(AIR, SAND, distance <= min(min(extents.x, extents.y), extents.z));
    // buffer[index].velocity.y = 69.;
}

fn index(position: vec3<u32>) -> u32 {
    return position.x + position.y * uniforms.width + position.z * uniforms.width * uniforms.height;
}

@compute
@workgroup_size(1)
fn simulate(@builtin(global_invocation_id) position: vec3<u32>) {
    // buffer[index(position)].material = AIR;
    let dimensions = vec3<u32>(uniforms.width, uniforms.height, uniforms.depth);
    let cell = buffer[index(position)];
    if cell.material == SAND {
        // buffer[index(position)].material = AIR;
        let nextPosition = vec3<u32>(vec3<i32>(position) + vec3<i32>(0, -1, 0));
        if all(nextPosition >= vec3<u32>(0u, 0u, 0u)) && all(nextPosition < dimensions) {
            let nextCell = buffer[index(nextPosition)];
            if nextCell.material == AIR {
                buffer[index(nextPosition)].material = SAND;
                buffer[index(position)].material = AIR;
            }
        }
    }
}