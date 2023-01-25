use std::path::Path;

mod default_impls;
pub use default_impls::*;

use crate::error::IndigoError;

pub trait FromIndigoMesh {
    fn convert<T: IndigoVertex>(indigo_mesh: &impl IndigoMesh<Vertex = T>) -> Self;
}

pub trait FromIndigoUniform {
    fn convert(indigo_uniform: &impl IndigoUniform) -> Self;
}

//Idk about this, limiting types sucks but making all types available would make a giant conversion
//method in the renderer impl
pub enum VertexType {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
}

pub trait IndigoVertex: bytemuck::Pod + bytemuck::Zeroable {
    fn vertex_layout() -> Vec<VertexType>;
}

pub trait IndigoMesh {
    type Vertex: IndigoVertex;
    fn vertices(&self) -> Vec<Self::Vertex>;
    fn indices(&self) -> Vec<u16>;
}

pub trait IndigoUniform: bytemuck::Pod + bytemuck::Zeroable {}

pub trait IndigoRenderCommand {
    type Mesh;
    type Uniform;
    type ShaderHandle;
    type TextureHandle;

    fn new(mesh: Self::Mesh, shader: Self::ShaderHandle) -> Self;
    fn add_uniform(&mut self, uniform: Self::Uniform);
    fn add_texture(&mut self, texture: Self::TextureHandle);
}

pub trait IndigoRenderer {
    type ErrorMessage: std::fmt::Debug;

    //Constrain mesh and uniform for the default renderer so custom views/apps/widgets
    //dont have to specify a concrete renderer type/dont have to add these constraints themselves
    #[cfg(feature = "wgpu-renderer")]
    type Mesh: FromIndigoMesh;
    #[cfg(feature = "wgpu-renderer")]
    type Uniform: FromIndigoUniform;

    //If the default renderer is not being used then these can be unconstrained in case
    //default widgets are not desired (they still can use them if their mesh/uniform types
    //implement FromIndigoMesh and FromIndigoUniform respectively)
    #[cfg(not(feature = "wgpu-renderer"))]
    type Mesh;
    #[cfg(not(feature = "wgpu-renderer"))]
    type Uniform;

    type TextureHandle;
    type ShaderHandle;

    // Only force IndigoRenderCommand constraint for the default renderer
    // a custom renderer's rendercommand would have to implement it anyway in order to
    #[cfg(feature = "wgpu-renderer")]
    type RenderCommand: IndigoRenderCommand<
        Mesh = Self::Mesh,
        Uniform = Self::Uniform,
        TextureHandle = Self::TextureHandle,
        ShaderHandle = Self::ShaderHandle,
    >;

    #[cfg(not(feature = "wgpu-renderer"))]
    type RenderCommand;

    fn render(
        &mut self,
        render_commands: Vec<Self::RenderCommand>,
    ) -> Result<(), IndigoError<Self::ErrorMessage>>;

    fn setup_camera(
        &mut self,
        pos: [f32; 3],
        target: [f32; 3],
        up: [f32; 3],
        znear: f32,
        zfar: f32,
    );

    fn on_window_resize(&mut self, new_window_size: (u32, u32));

    fn fetch_texture(&mut self, texture_path: &Path) -> Self::TextureHandle;
    fn fetch_shader(
        &mut self,
        vertex_shader: &str,
        vertex_entry: &str,
        fragment_shader: &str,
        fragment_entry: &str,
    ) -> Self::ShaderHandle;
    fn get_camera_uniform(&self) -> Self::Uniform;
}

#[cfg(feature = "wgpu-renderer")]
pub use wgpu_renderer_glue::*;

#[cfg(feature = "wgpu-renderer")]
mod wgpu_renderer_glue {

    use std::path::{Path, PathBuf};

    use crate::{error::IndigoError};
    use ahash::AHashMap;
    pub use indigo_wgpu::*;
    use indigo_wgpu::{
        camera::Camera,
        mesh::{VertexLayoutInfo, Mesh},
        shader::Shader,
        wgpu::{VertexFormat, SurfaceError},
    };
    use winit::window::Window;

    use super::{FromIndigoMesh, FromIndigoUniform, IndigoRenderCommand, IndigoRenderer};

    pub struct IndigoWgpuError(indigo_wgpu::wgpu::SurfaceError);

    pub struct WgpuRenderer {
        context: renderer::RenderingContext,
        main_camera: Option<camera::Camera>,
    }

    impl WgpuRenderer {
        pub async fn new(window: &Window) -> Self {
            let context =
                indigo_wgpu::renderer::RenderingContext::new(window.inner_size(), window).await;

            Self {
                context,
                main_camera: None,
            }
        }
    }

    impl FromIndigoMesh for Mesh {
        fn convert<T: super::IndigoVertex>(
            indigo_mesh: &impl super::IndigoMesh<Vertex = T>,
        ) -> Self {
            let attributes = T::vertex_layout()
                .into_iter()
                .map(|vtype| match vtype {
                    super::VertexType::Float32 => VertexFormat::Float32,
                    super::VertexType::Float32x2 => VertexFormat::Float32x2,
                    super::VertexType::Float32x3 => VertexFormat::Float32x3,
                    super::VertexType::Float32x4 => VertexFormat::Float32x4,
                    super::VertexType::Uint32 => VertexFormat::Uint32,
                    super::VertexType::Uint32x2 => VertexFormat::Uint32x2,
                    super::VertexType::Uint32x3 => VertexFormat::Uint32x3,
                    super::VertexType::Uint32x4 => VertexFormat::Uint32x4,
                    super::VertexType::Sint32 => VertexFormat::Sint32,
                    super::VertexType::Sint32x2 => VertexFormat::Sint32x2,
                    super::VertexType::Sint32x3 => VertexFormat::Sint32x3,
                    super::VertexType::Sint32x4 => VertexFormat::Sint32x4,
                })
                .scan(0, |offset, vformat| {
                    *offset += vformat.size();
                    Some((*offset, vformat))
                })
                .enumerate()
                .map(|(index, (offset, vformat))| {
                    wgpu::VertexAttribute {
                        // Need to subtract the current format size in order to for the offsets
                        // to start at zero
                        offset: offset - vformat.size(),
                        shader_location: index as u32,
                        format: vformat,
                    }
                })
                .collect::<Vec<_>>();

            Self {
                vertices: bytemuck::cast_slice(indigo_mesh.vertices().as_slice()).to_vec(),
                indices: indigo_mesh.indices(),
                layout: VertexLayoutInfo {
                    array_stride: std::mem::size_of::<T>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes,
                },
            }
        }
    }

    impl FromIndigoUniform for (Vec<u8>, wgpu::ShaderStages) {
        fn convert(indigo_uniform: &impl super::IndigoUniform) -> Self {
            //hardcoded shader stage for now
            (
                bytemuck::cast_slice(&[*indigo_uniform]).to_vec(),
                wgpu::ShaderStages::VERTEX,
            )
        }
    }

    pub struct WgpuRenderCommand<M, U, S, T> {
        pub mesh: M,
        pub shader: S,
        pub textures: Vec<T>,
        pub uniforms: Vec<U>,
    }

    impl<M: FromIndigoMesh, U: FromIndigoUniform, S, T> IndigoRenderCommand
        for WgpuRenderCommand<M, U, S, T>
    {
        type Mesh = M;
        type Uniform = U;
        type ShaderHandle = S;
        type TextureHandle = T;

        fn new(mesh: Self::Mesh, shader: Self::ShaderHandle) -> Self {
            Self {
                mesh,
                shader,
                textures: Vec::new(),
                uniforms: Vec::new(),
            }
        }

        fn add_texture(&mut self, texture: Self::TextureHandle) {
            self.textures.push(texture);
        }

        fn add_uniform(&mut self, uniform: Self::Uniform) {
            self.uniforms.push(uniform);
        }
    }

    impl IndigoRenderer for WgpuRenderer {
        type ErrorMessage = String;
        type Mesh = Mesh;
        type Uniform = (Vec<u8>, wgpu::ShaderStages);
        type TextureHandle = PathBuf;
        type ShaderHandle = Shader;

        type RenderCommand =
            WgpuRenderCommand<Self::Mesh, Self::Uniform, Self::ShaderHandle, Self::TextureHandle>;

        fn render(
            &mut self,
            render_commands: Vec<Self::RenderCommand>,
        ) -> Result<(), IndigoError<Self::ErrorMessage>> {
            #[derive(Eq, PartialEq, Hash, Clone)]
            struct BatchInfo {
                layout: VertexLayoutInfo,
                shader: <WgpuRenderer as IndigoRenderer>::ShaderHandle,
                textures: Vec<<WgpuRenderer as IndigoRenderer>::TextureHandle>,
                uniforms: Vec<<WgpuRenderer as IndigoRenderer>::Uniform>,
            }

            let mut batches = AHashMap::<BatchInfo, Vec<Mesh>>::new();

            for command in render_commands {
                let Self::RenderCommand {
                    mesh,
                    shader,
                    textures,
                    uniforms,
                } = command;

                //Find a way to group the commands without cloning everything
                let batch_info = BatchInfo {
                    layout: mesh.layout.clone(),
                    shader,
                    textures,
                    uniforms,
                };

                match batches.get_mut(&batch_info) {
                    Some(commands) => {
                        commands.push(mesh);
                    }
                    None => {
                        batches.insert(batch_info, vec![mesh]);
                    }
                };
            }

            let output_res = self.context.surface.get_current_texture();

            if let Err(SurfaceError::Outdated) = output_res {
                return Err(IndigoError::FatalError { msg: "surface outdated".to_owned() });
            } else if let Ok(output) = output_res {

                let output_tex = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
        
    
                for (key, batch) in batches.drain() {
                    let mesh = batch
                        .into_iter()
                        .reduce(|mut main, mut mesh| {
                            main.merge(&mut mesh);
                            main
                        })
                        .unwrap();
    
                    self.context.batch_render(
                        mesh,
                        key.shader,
                        key.textures,
                        key.uniforms,
                        &output_tex,
                    ) 
                }
    
                output.present();
            }

            Ok(())
        }

        fn on_window_resize(&mut self, new_window_size: (u32, u32)) {
            self.context.update_surface(new_window_size);
            self.main_camera.as_mut().unwrap().update(new_window_size);
        }

        fn setup_camera(
            &mut self,
            pos: [f32; 3],
            target: [f32; 3],
            up: [f32; 3],
            znear: f32,
            zfar: f32,
        ) {
            let mut camera = Camera::new(pos.into(), target.into(), up.into(), znear, zfar);

            camera.update((self.context.config.width, self.context.config.height));

            self.main_camera = Some(camera);
        }

        fn fetch_texture(&mut self, texture_path: &std::path::Path) -> Self::TextureHandle {
            self.context.create_texture_if_doesnt_exist(texture_path);
         
            texture_path.to_path_buf()
        }

        fn fetch_shader(
            &mut self,
            vertex_shader: &str,
            vertex_entry: &str,
            fragment_shader: &str,
            fragment_entry: &str,
        ) -> Self::ShaderHandle {
            self.context
                .create_shader_module_if_doesnt_exist(vertex_shader);
            self.context
                .create_shader_module_if_doesnt_exist(fragment_shader);

            //Probably not too efficient considering all these copies for every
            //submitted render command with this shader
             Shader::new(
                vertex_shader,
                vertex_entry.to_owned(),
                fragment_shader,
                fragment_entry.to_owned(),
            )
        }

        fn get_camera_uniform(&self) -> Self::Uniform {
            let camera_uniform = self.main_camera.as_ref().unwrap().uniform;
            let data = bytemuck::cast_slice(&[camera_uniform]).to_vec();
            (data, wgpu::ShaderStages::VERTEX)
        }
    }
}
