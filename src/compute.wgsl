struct Ruleset {
  ruleset : array<u32, 27u>;
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

let GRID_WIDTH: i32 = 40;

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
        if (nx < 0 || nx > GRID_WIDTH - 1 || ny < 0 || ny > GRID_WIDTH - 1 || nz < 0 || nz > GRID_WIDTH - 1) {
          continue;
        }

        let neighbour_state = cellsSrc.cells[nz + (ny * GRID_WIDTH) + (nx * GRID_WIDTH * GRID_WIDTH)].state;
        if (neighbour_state == ALIVE_STATE) {
          neighbour_count = neighbour_count + 1;
        } 
      }
    }
  }


  if (ruleset.ruleset[neighbour_count] == SURVIVE_RULE && cell.state == ALIVE_STATE) { // Stay alive
    cellsDst.cells[index].state = DEAD_STATE;
  } else if (ruleset.ruleset[neighbour_count] == BIRTH_RULE && cell.state == DEAD_STATE) { // Become alive
    cellsDst.cells[index].state = ALIVE_STATE;
  } else if (ruleset.ruleset[neighbour_count] == DEATH_RULE)  {
    cellsDst.cells[index].state = DEAD_STATE;
  }
}
