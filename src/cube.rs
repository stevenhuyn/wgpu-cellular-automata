use crate::scene::Vertex;

pub struct Cube {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Cube {
    pub fn new(x: f32, y: f32, z: f32, width: f32, color: [f32; 3]) -> Self {
        let mut vertices = Vec::new();

        for dx in 0..2 {
            for dy in 0..2 {
                for dz in 0..2 {
                    let vx = x + dx as f32 * width;
                    let vy = y + dy as f32 * width;
                    let vz = z + dz as f32 * width;
                    let position = [vx, vy, vz];
                    vertices.push(Vertex { position, color })
                }
            }
        }

        #[rustfmt::skip]
        let indices = vec![
            0, 2, 3,
            0, 3, 1,
            0, 6, 2,
            0, 4, 6,
            0, 1, 5,
            0, 5, 4,
            7, 3, 2,
            7, 2, 6,
            7, 4, 5,
            7, 6, 4,
            7, 5, 1,
            7, 1, 3
        ];

        Self {
            x,
            y,
            z,
            vertices,
            indices,
        }
    }
}
