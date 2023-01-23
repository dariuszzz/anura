#[derive(Clone, Eq, PartialEq, Hash)]
pub struct LayoutInfo {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<u8>,
    pub indices: Vec<u16>,
    pub layout: LayoutInfo,
}

impl Mesh {
    pub fn descriptor<'a>(&'a self) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: self.layout.array_stride,
            step_mode: self.layout.step_mode,
            attributes: &self.layout.attributes
        }
    }
    
    pub fn merge(&mut self, other: &mut Mesh) {
        other.indices.iter_mut().for_each(|i| {
            *i += self.vertices.len() as u16;
        });

        self.indices.append(&mut other.indices);
        self.vertices.append(&mut other.vertices);
    }
}