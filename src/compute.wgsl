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
        let width = 15;

        // Checking bounds of the grid
        if (nx < 0 || nx > width - 1 || ny < 0 || ny > width - 1 || nz < 0 || nz > width - 1) {
          continue;
        }

        let neighbour_state = cellsSrc.cells[nz + (ny * width) + (nx * width * width)].state;
        if (neighbour_state == 1) {
          neighbour_count = neighbour_count + 1;
        } 
      }
    }
  }


  if (ruleset.ruleset[neighbour_count] == 1u && cell.state == 1) { // Stay alive
    cellsDst.cells[index].state = 0;
  } else if (ruleset.ruleset[neighbour_count] == 2u && cell.state == 1) { // Become alive
    cellsDst.cells[index].state = 1;
  } else if (ruleset.ruleset[neighbour_count] == 0u)  {
    cellsDst.cells[index].state = 0;
  }
}
