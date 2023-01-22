use super::{IndigoVertex, VertexType, IndigoMesh, IndigoRenderCommand, FromIndigoMesh};




#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Default)]
pub struct DefaultVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl IndigoVertex for DefaultVertex {
    fn vertexLayout() -> Vec<VertexType> {
        vec![
            VertexType::Float32x3,
            VertexType::Float32x4, 
            VertexType::Float32x2,
        ]
    }
}

pub struct DefaultMesh<V> {
    pub verts: Vec<V>,
    pub inds: Vec<u16>
}

impl<V: IndigoVertex> IndigoMesh for DefaultMesh<V> {
    type Vertex = V;

    fn indices(&self) -> Vec<u16> {
        self.inds.clone()
    }

    fn vertices(&self) -> Vec<Self::Vertex> {
        self.verts.clone()
    }
}

impl DefaultMesh<DefaultVertex> {
    pub fn quad(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            verts: vec![
                DefaultVertex { pos: [x, y, 0.0], color: [0.0, 0.0, 0.0, 1.0], uv: [0.0, 0.0] },
                DefaultVertex { pos: [w, y, 0.0], color: [1.0, 0.0, 0.0, 1.0], uv: [1.0, 0.0] },
                DefaultVertex { pos: [w, h, 0.0], color: [0.0, 1.0, 0.0, 1.0], uv: [1.0, 1.0] },
                DefaultVertex { pos: [x, h, 0.0], color: [0.0, 0.0, 1.0, 1.0], uv: [0.0, 1.0] },
            ],
            inds: vec![
                0, 1, 2, 2, 3, 0
            ]
        }
    }
}

