#import ../common.wgsl as Common

alias Particle = Common::Particle;
alias Bounds = Common::Bounds;
alias GridCell = Common::GridCell;

const GRID_SIZE = Common::GRID_SIZE;
const MAX_PARTICLES_PER_GRID_CELL = Common::MAX_PARTICLES_PER_GRID_CELL;

@group(0)
@binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0)
@binding(1)
var<storage, read> bounds: array<Bounds>;

@group(0)
@binding(2)
var<storage, read> grid: array<GridCell>;

@compute
@workgroup_size(1)
fn simulate(@builtin(global_invocation_id) position: vec3<u32>) {
}