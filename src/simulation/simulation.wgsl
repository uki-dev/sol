const AIR = 0u;
const SAND = 1u;
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
var<storage, read_write> cells: array<Cell>;

fn index_1d(position: vec3<u32>) -> u32 {
    return position.x + position.y * uniforms.width + position.z * uniforms.width * uniforms.height;
}

@compute
@workgroup_size(1)
fn populate(@builtin(global_invocation_id) position: vec3<u32>) {
    let extents = vec3<f32>(vec3<u32>(uniforms.width, uniforms.height, uniforms.depth)) * .5;
    let distance = length(vec3<f32>(position) - extents + 0.5);
    let index = index_1d(position);
    cells[index].material = select(AIR, SAND, distance <= min(min(extents.x, extents.y), extents.z));
}

@compute
@workgroup_size(1)
fn simulate(@builtin(global_invocation_id) position: vec3<u32>) {
    let index = index_1d(position);
    let cell = cells[index];
    if cell.material == SAND {
        sand(position, index);
    }
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
    // TODO: can we do this casting etc outside the shader invocation?
    let dimensions = vec3<i32>(vec3<u32>(uniforms.width, uniforms.height, uniforms.depth));
    for (var i = 0; i < 9; i += 1) {
        let neighbour_position = vec3<i32>(position) + neighbours[i];
        if all(neighbour_position >= vec3<i32>(0, 0, 0)) && all(neighbour_position < dimensions) {
            let neighbour_index = index_1d(vec3<u32>(neighbour_position));
            let neighbour = cells[neighbour_index];
            if neighbour.material == AIR {
                cells[neighbour_index].material = SAND;
                cells[index].material = AIR;
                return;
            }
        }
    }
}

const SPHERE = 0u;
struct Object {
    matrix: mat4x4<f32>,
    colour: vec4<f32>,
    sdf: u32,
}

// TODO: move into a separate layout group specific for mapping to objects
// or abstract this logic from this implementation
@group(0)
@binding(2)
var<storage, read_write> objects: array<Object>;

@group(0)
@binding(3)
var<storage, read_write> objects_length: atomic<u32>;

@compute
@workgroup_size(1)
fn map_cells_to_objects(@builtin(global_invocation_id) position: vec3<u32>) {
    let cell = cells[index_1d(position)];
    if (cell.material != AIR) {
        let dimensions = vec3<i32>(vec3<u32>(uniforms.width, uniforms.height, uniforms.depth));

        var neighbours: array<vec3<i32>, 6> = array<vec3<i32>, 6>(
            vec3<i32>(-1, 0, 0),
            vec3<i32>(1, 0, 0),
            vec3<i32>(0, -1, 0),
            vec3<i32>(0, 1, 0),
            vec3<i32>(0, 0, -1),
            vec3<i32>(0, 0, 1),
        );
        var visible = false;
        for (var i = 0; i < 6; i += 1) {
            // TODO: ensure that cast does not ceil negative to zero because this would pass check when it should not
            let neighbour_position = vec3<i32>(position) + neighbours[i];
            if all(neighbour_position >= vec3<i32>(0, 0, 0)) && all(neighbour_position < dimensions) {
                let neighbour_index = index_1d(vec3<u32>(neighbour_position));
                let neighbour = cells[neighbour_index];
                if neighbour.material == AIR {
                    visible = true;
                    break;
                }
            } else {
                visible = true;
                break;
            }
        }
        if !visible {
            return;
        }

        var object: Object;
        let offset = vec3<f32>(dimensions) * 0.5 - 0.5;
        object.matrix = mat4x4<f32>(
            vec4<f32>(1., 0., 0., 0.),
            vec4<f32>(0., 1., 0., 0.),
            vec4<f32>(0., 0., 1., 0.),
            // TODO: add jitter
            // TODO: can multiply this by a world matrix for the entire grid?
            vec4<f32>(vec3<f32>(position) - offset, 1.)
        );
        object.sdf = SPHERE;
        object.colour = vec4<f32>(.9, .7, .3, 1.);
        let index = atomicAdd(&objects_length, 1u);
        objects[index] = object;
    }
}