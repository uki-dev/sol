const AIR = 0u;
const WATER = 1u;
const SAND = 2u;
const SOIL = 3u;

struct Cell {
    material: u32,
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
}

@compute
@workgroup_size(1)
fn simulate(@builtin(global_invocation_id) position: vec3<u32>) {
    let index = index(position);
    let cell = buffer[index];
    if cell.material == SAND {
        sand(position, index);
    }
}

fn index(position: vec3<u32>) -> u32 {
    return position.x + position.y * uniforms.width + position.z * uniforms.width * uniforms.height;
}

fn sand(position: vec3<u32>, index: u32) {
    var neighbours: array<vec3<i32>, 9> = array<vec3<i32>, 9>(
        vec3<i32>(0, -1, 0),
        vec3<i32>(-1, -1, 0),
        vec3<i32>(1, -1, 0),
        vec3<i32>(0, -1, -1),
        vec3<i32>(0, -1, 1),
        vec3<i32>(-1, -1, -1),
        vec3<i32>(1, -1, 1),
        vec3<i32>(1, -1, -1),
        vec3<i32>(1, -1, 1),
    );

    let dimensions = vec3<u32>(uniforms.width, uniforms.height, uniforms.depth);
    for (var i = 0; i < 9; i += 1) {
        let neighbour_position = vec3<u32>(vec3<i32>(position) + neighbours[i]);
        if all(neighbour_position >= vec3<u32>(0u, 0u, 0u)) && all(neighbour_position < dimensions) {
            let neighbour_index = index(neighbour_position);
            let neighbour = buffer[index(neighbour_position)];
            if neighbour.material == AIR {
                buffer[neighbour_index].material = SAND;
                buffer[index].material = AIR;
                return;
            }
        }
    }
}
