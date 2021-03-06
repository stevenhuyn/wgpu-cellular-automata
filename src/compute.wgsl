struct Ruleset {
  ruleset : array<u32, 27u>;
  grid_width: u32;
};

struct Cell {
  state : i32;
  x     : i32;
  y     : i32;
  z     : i32;
};

struct Cells {
  cells : [[stride(16)]] array<Cell>;
};

let DEAD_STATE: i32 = 0;
let ALIVE_STATE: i32 = 1;

let DEATH_RULE = 0u;
let SURVIVE_RULE = 1u;
let BIRTH_RULE = 2u;


[[group(0), binding(0)]] var<uniform> ruleset : Ruleset;
[[group(0), binding(1)]] var<storage, read> cellsSrc : Cells;
[[group(0), binding(2)]] var<storage, read_write> cellsDst : Cells;

[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let index = global_invocation_id.x;
  var neighbour_count = 0;

  // TODO: Is casting expensive?? Should really just use an encase or crevise
  let grid_width = i32(ruleset.grid_width); 

  let cell = cellsSrc.cells[index];
  for (var dx = -1; dx < 2; dx = dx + 1) {
    for (var dy = -1; dy < 2; dy = dy + 1) {
      for (var dz = -1; dz < 2; dz = dz + 1) {
        if (dx == 0 && dy == 0 && dz == 0) {
          continue;
        }

        // Getting candidate neighbor
        let nx = cell.x + dx;
        let ny = cell.y + dy;
        let nz = cell.z + dz;

        // Checking bounds of the grid
        if (nx < 0 || nx > grid_width - 1 || ny < 0 || ny > grid_width - 1 || nz < 0 || nz > grid_width - 1) {
          continue;
        }

        let neighbour_state = cellsSrc.cells[nz + (ny * grid_width) + (nx * grid_width * grid_width)].state;
        if (neighbour_state == ALIVE_STATE) {
          neighbour_count = neighbour_count + 1;
        } 
      }
    }
  }


  let rule = ruleset.ruleset[neighbour_count];
  if (rule == SURVIVE_RULE && cell.state == ALIVE_STATE) {
    cellsDst.cells[index].state = DEAD_STATE;
  } else if (rule == BIRTH_RULE && cell.state == DEAD_STATE) {
    cellsDst.cells[index].state = ALIVE_STATE;
  } else {
    cellsDst.cells[index].state = DEAD_STATE;
  }
}
