struct Ruleset {
  ruleset : array<u32, 28u>;
};

struct Cell {
  state : i32;
  pos   : vec3<i32>;
};

struct Cells {
  cells : [[stride(32)]] array<Cell>;
};

[[group(0), binding(0)]] var<uniform> ruleset : Ruleset;
[[group(0), binding(1)]] var<storage, read> cellsSrc : Cells;
[[group(0), binding(2)]] var<storage, read_write> cellsDst : Cells;

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let index = global_invocation_id.x;
  var neighbour_count = 0;
  let cell_pos = cellsSrc.cells[index].pos;
  for (var dx = -1; dx < 2; dx = dx + 1) {
    for (var dy = -1; dy < 2; dy = dy + 1) {
      for (var dz = -1; dz < 2; dz = dz + 1) {
        // Getting candidate neighbor
        let nx = cell_pos.x + dx;
        let ny = cell_pos.y + dy;
        let nz = cell_pos.z + dz;

        // Checking bounds of the grid
        if (nx < 0 || nx > 8 || ny < 0 || ny > 8 || nz < 0 || nz > 8) {
          continue;
        }

        let neighbour_state = cellsSrc.cells[nx + (ny * 5) + (nz * 25)].state;
        if (neighbour_state == 1) {
          neighbour_count = neighbour_count + 1;
        } 
      }
    }
  }


  if (ruleset.ruleset[neighbour_count] == 0u) { // Become dead
    cellsDst.cells[index].state = 0;
  } else if (ruleset.ruleset[neighbour_count] == 2u) { // Become alive
    cellsDst.cells[index].state = 1;
  } else {
    cellsDst.cells[index].state = 0;
  }

}
