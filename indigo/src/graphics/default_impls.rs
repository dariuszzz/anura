use super::{
    IndigoMesh,
    IndigoVertex, VertexType,
};

//Embed the default shaders :)
pub static PLAIN_SHADER: &str = include_str!("../shaders/plain.wgsl");
pub static IMAGE_SHADER: &str = include_str!("../shaders/image.wgsl");

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Default)]
pub struct DefaultVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl IndigoVertex for DefaultVertex {
    fn vertex_layout() -> Vec<VertexType> {
        vec![
            VertexType::Float32x3,
            VertexType::Float32x4,
            VertexType::Float32x2,
        ]
    }
}

pub struct DefaultMesh<V> {
    pub verts: Vec<V>,
    pub inds: Vec<u16>,
    pub could_be_transparent: bool,
    pub highest_z: f32
}

impl<V: IndigoVertex> IndigoMesh for DefaultMesh<V> {
    type Vertex = V;

    fn indices(&self) -> Vec<u16> {
        self.inds.clone()
    }

    fn vertices(&self) -> Vec<Self::Vertex> {
        self.verts.clone()
    }

    fn could_be_transparent(&self) -> bool {
        self.could_be_transparent
    }

    fn highest_z(&self) -> f32 {
        self.highest_z
    }
}

impl DefaultMesh<DefaultVertex> {
    pub fn quad(pos: (f32, f32, f32), dim: (f32, f32)) -> Self {
        let (x, y, z) = pos;
        let (w, h) = dim;

        Self {

            verts: vec![
                DefaultVertex {
                    pos: [x, y, z],
                    color: [0.0, 0.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                },                
                DefaultVertex {
                    pos: [x, y + h, z],
                    color: [0.0, 0.0, 1.0, 1.0],
                    uv: [0.0, 1.0],
                },
                DefaultVertex {
                    pos: [x + w, y + h, z],
                    color: [0.0, 1.0, 0.0, 1.0],
                    uv: [1.0, 1.0],
                },
                DefaultVertex {
                    pos: [x + w, y, z],
                    color: [1.0, 0.0, 0.0, 1.0],
                    uv: [1.0, 0.0],
                },
            ],
            inds: vec![0, 1, 2, 2, 3, 0],
            could_be_transparent: false,
            highest_z: z,
        }
    }

    pub fn possibly_trasparent(&mut self) {
        self.could_be_transparent = true;
    }
}
