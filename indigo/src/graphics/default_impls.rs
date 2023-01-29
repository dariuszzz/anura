use std::default;

use crate::font::{FontAtlas, GlyphData};

use super::{
    IndigoMesh,
    IndigoVertex, VertexType, IndigoRenderer,
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
    pub fn quad(pos: (f32, f32, f32), dim: (f32, f32), uv: (f32, f32, f32, f32), color: (f32, f32, f32, f32)) -> Self {
        let (x, y, z) = pos;
        let (w, h) = dim;
        let (uv_x, uv_y, uv_w, uv_h) = uv;
        let (r, g, b, a) = color;

        Self {

            verts: vec![
                DefaultVertex {
                    pos: [x, y, z],
                    color: [r, g, b, a],
                    uv: [uv_x, uv_y],
                },                
                DefaultVertex {
                    pos: [x, y + h, z],
                    color: [r, g, b, a],
                    uv: [uv_x, uv_y + uv_h],
                },
                DefaultVertex {
                    pos: [x + w, y + h, z],
                    color: [r, g, b, a],
                    uv: [uv_x + uv_w, uv_y + uv_h],
                },
                DefaultVertex {
                    pos: [x + w, y, z],
                    color: [r, g, b, a],
                    uv: [uv_x + uv_w, uv_y],
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

    pub fn merge(&mut self, other: &mut Self) {
        other.inds.iter_mut().for_each(|i| {
            *i += self.verts.len() as u16;
        });

        // deconstructed triangle between meshes
        if let Some(last_index_from_other) = other.inds.last().cloned() {
            if let Some(last_self_index) = self.inds.last().cloned() {
                self.inds.append(&mut vec![
                    last_self_index,
                    last_self_index,
                    last_index_from_other,
                ]);
            }
        }

        self.inds.append(&mut other.inds);
        self.verts.append(&mut other.verts);

        if other.highest_z > self.highest_z { 
            self.highest_z = other.highest_z;
        }

        if other.could_be_transparent {
            self.could_be_transparent = true;
        }
    }

    
    pub fn bounded_text<R: IndigoRenderer>(
        initial_pos: (f32, f32, f32),
        bounds: Option<(f32, f32)>,
        text: &str,
        font: &FontAtlas<R>,
    ) -> Self {
        let line_spacing = 5.0;

        let mut pos = (
            initial_pos.0,
            initial_pos.1 + font.size, //Set baseline
            initial_pos.2,
        );

        let mut mesh = Self {
            verts: Vec::new(),
            inds: Vec::new(),
            could_be_transparent: true,
            highest_z: pos.2
        };

        for c in text.chars() {
            //Skip newlines and move to new line
            if c == '\r' || c == '\n' {
                pos.0 = initial_pos.0;
                pos.1 += font.size + line_spacing;

                continue;
            }

            let GlyphData { uv, metrics } = match font.glyph_data.get(&c) {
                Some(gd) => gd,
                None => continue,
            };

            let mut dim = (metrics.width as f32, (metrics.height as f32));

            let mut current_pos = (
                pos.0 + metrics.xmin as f32,
                pos.1 - (dim.1 + metrics.ymin as f32),
                pos.2 + 1.0,
            );

            //handle overflow
            if let Some(bounds) = bounds {
                if current_pos.0 + dim.0 > initial_pos.0 + bounds.0 {
                    current_pos.1 += font.size + line_spacing;
                    current_pos.0 = initial_pos.0 + metrics.xmin as f32;
                    pos.1 += font.size + line_spacing;
                    pos.0 = current_pos.0;
                }

                if current_pos.1 + (dim.1 + metrics.ymin as f32) > initial_pos.1 + bounds.1 + metrics.ymin as f32 {
                    break;
                }
            }

            if c == ' ' {
                dim.0 += metrics.advance_width;
            }

            mesh.merge(&mut Self::quad(
                current_pos,
                dim,
                (uv.0, uv.1, uv.2, uv.3),
                (0.0, 0.0, 0.0, 1.0),
            ));

            pos.0 += metrics.advance_width;
        }

        mesh
    }
}
