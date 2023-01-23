use crate::uniform::UniformBinding;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

pub struct Camera {
    pub pos: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub znear: f32,
    pub zfar: f32,

    pub uniform: CameraUniform,
}

impl Camera {
    pub fn update(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.uniform.view_proj = self.build_view_proj_matrix(config).into();
    }

    pub fn build_view_proj_matrix(&self, config: &wgpu::SurfaceConfiguration) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.pos, self.target, self.up);

        let scale_x = config.width as f32;
        let scale_y = config.height as f32;

        let proj = cgmath::ortho(0.0, scale_x, scale_y, 0.0, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
//     pub fn new(
//         device: &wgpu::Device,
//         pos: cgmath::Point3<f32>,
//         target: cgmath::Point3<f32>,
//         up: cgmath::Vector3<f32>,
//         znear: f32,
//         zfar: f32
//     ) -> Self {
//         let uniform = CameraUniform::new();

//         let buffer = device.create_buffer_init(
//             &wgpu::util::BufferInitDescriptor {
//                 label: Some("Camera buffer"),
//                 contents: bytemuck::cast_slice(&[uniform]),
//                 usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//             }
//         );

//         let bg_layout = device.create_bind_group_layout(
//             &wgpu::BindGroupLayoutDescriptor {
//                 label: Some("OrthoCamera bgl"),
//                 entries: &[
//                     wgpu::BindGroupLayoutEntry {
//                         binding: 0,
//                         visibility: wgpu::ShaderStages::VERTEX,
//                         ty: wgpu::BindingType::Buffer {
//                             ty: wgpu::BufferBindingType::Uniform,
//                             has_dynamic_offset: false,
//                             min_binding_size: None
//                         },
//                         count: None
//                     }
//                 ]
//             }
//         );  

//         let bind_group = device.create_bind_group(
//             &wgpu::BindGroupDescriptor {
//                 layout: &bg_layout,
//                 entries: &[
//                     wgpu::BindGroupEntry {
//                         binding: 0,
//                         resource: buffer.as_entire_binding()
//                     }
//                 ],
//                 label: Some("Ortho bind group")
//             }
//         );

//         Self {
//             pos,
//             target,
//             up,
//             zfar,
//             znear,
//             uniform,
//             buffer,
//             bg_layout,
//             bind_group
//         }
//     }
// }