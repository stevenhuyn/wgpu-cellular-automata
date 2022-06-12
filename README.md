# 3D Cellular Automata via wgpu

Making a 3D Celluar Automata (Like Conway's Game but 3D!) to learn [wgpu](https://github.com/gfx-rs/wgpu) a cross-plaform graphics API.

Foundations of the code built off of [sotrh's tutorial](https://sotrh.github.io/learn-wgpu/#what-is-wgpu)

Used [Chris Evan's](https://chrisevans9629.github.io/blog/2020/07/27/game-of-life) blog post as correctness reference.

## Running 

Show simulation with default grid width of 30
```
cargo run --release
```

See options:
```
cargo run --release -- --help
```

## Todo
- [x] Cubes
- [x] Compute Shader
- [ ] Optimise Vertex Buffer/Index Buffer Generation (?)
- [ ] Better Camera Controls
- [ ] Configuration (camera poistion, ruleset, grid width)
- [ ] Aesthetic changes
  - [ ]  Lighting,
  - [ ]  Birth/Death animation
  - [x]  Anti-aliasing via [smaa-rs](https://github.com/fintelia/smaa-rs)
- [ ] WebGPU example
- [ ] Make transparent high quality GIF

## Showcase

<p align="center">
  <img src="./media/conwaycorrect.gif" width="auto">
</p>
