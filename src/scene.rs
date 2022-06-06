use crate::cube::Cube;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    // TODO: Make a new function

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Scene {
    pub cubes: Vec<Cube>,
}

impl Scene {
    pub fn new() -> Self {
        Self { cubes: Vec::new() }
    }

    pub fn new_basic_scene() -> Self {
        let cubes = vec![
            Cube::new(0., 0., 0., 1., [1., 0., 0.]),
            Cube::new(1., 0., 0., 1., [0., 1., 0.]),
            Cube::new(0., 1., 0., 1., [0., 0., 1.]),
            Cube::new(0., 1., 1., 1., [1., 1., 0.5]),
        ];
        Self { cubes }
    }

    pub fn add_cube(&mut self, cube: Cube) {
        self.cubes.push(cube);
    }

    pub fn get_vertices_and_indices(&mut self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut running_index = 0;
        for cube in self.cubes.iter() {
            vertices.extend(&cube.vertices);
            indices.extend(cube.indices.iter().map(|x| x + running_index));
            running_index += cube.vertices.len() as u16;
        }

        (vertices, indices)
    }
}
