use std::path::Path;

mod default_impls;
pub use default_impls::*;

use crate::error::IndigoError;

//For some reason this doesnt compile when indigo_mesh is &T where T: IndigoMesh<Vertex = V>
//so it will stay as &impl IndigoMesh<Vertex = V> for now 
pub trait FromIndigoMesh {
    fn convert<V: IndigoVertex>(indigo_mesh: &impl IndigoMesh<Vertex = V>) -> Self;
}

pub trait FromIndigoUniform {
    fn convert<T: IndigoUniform>(indigo_uniform: &T) -> Self;
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
    fn could_be_transparent(&self) -> bool;
    fn highest_z(&self) -> f32; 
}

pub enum IndigoShaderStage {
    Both,
    Vertex,
    Fragment,
}

pub trait IndigoUniform: bytemuck::Pod + bytemuck::Zeroable {
    const SHADER_STAGE: IndigoShaderStage; 
}

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

    fn new_texture(&mut self, data: &[u8], dimensions: (u32, u32), handle: Option<Self::TextureHandle>) -> Self::TextureHandle; 
    fn update_texture(&mut self, texture_handle: Self::TextureHandle, data: &[u8]);
    fn load_texture(&mut self, texture_path: &Path) -> Self::TextureHandle;
    fn remove_texture(&mut self, texture_handle: Self::TextureHandle);

    fn load_shader(
        &mut self,
        vertex_shader: &str,
        vertex_entry: &str,
        fragment_shader: &str,
        fragment_entry: &str,
    ) -> Self::ShaderHandle;
    
    fn camera_uniform(&self) -> Self::Uniform;
}

#[cfg(feature = "wgpu-renderer")]
pub use wgpu_renderer_glue::*;

#[cfg(feature = "wgpu-renderer")]
mod wgpu_renderer_glue {

    use std::{path::PathBuf};

    use crate::{error::IndigoError};
    use ahash::AHashMap;
    use image::GenericImageView;
    pub use indigo_wgpu::*;
    use indigo_wgpu::{
        camera::Camera,
        mesh::{VertexLayoutInfo, Mesh},
        shader::Shader,
        wgpu::VertexFormat, renderer::BatchInfo, texture::Texture,
    };
    use winit::window::Window;

    use super::{FromIndigoMesh, FromIndigoUniform, IndigoRenderCommand, IndigoRenderer, IndigoShaderStage};

    pub struct IndigoWgpuError(indigo_wgpu::wgpu::SurfaceError);

    pub struct WgpuRenderer {
        context: renderer::RenderingContext,
        texture_map: AHashMap<PathBuf, usize>,
        main_camera: Option<camera::Camera>,
    }

    impl WgpuRenderer {
        pub async fn new(window: &Window) -> Self {
            let context =
                indigo_wgpu::renderer::RenderingContext::new(window.inner_size(), window).await;

            Self {
                context,
                texture_map: AHashMap::new(),
                main_camera: None,
            }
        }
    }

    impl FromIndigoMesh for Mesh {
        fn convert<V: super::IndigoVertex>(
            indigo_mesh: &impl super::IndigoMesh<Vertex = V>,
        ) -> Self {
            let attributes = V::vertex_layout()
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
                    array_stride: std::mem::size_of::<V>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes,
                },
                could_be_transparent: indigo_mesh.could_be_transparent(),
                highest_z: indigo_mesh.highest_z()
            }
        }
    }

    impl FromIndigoUniform for (Vec<u8>, wgpu::ShaderStages) {
        fn convert<T: super::IndigoUniform>(indigo_uniform: &T) -> Self {
            //hardcoded shader stage for now
            let shader_stage = match T::SHADER_STAGE {
                IndigoShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
                IndigoShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
                IndigoShaderStage::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
            };

            (
                bytemuck::cast_slice(&[*indigo_uniform]).to_vec(),
                shader_stage,
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
        type TextureHandle = usize;
        type ShaderHandle = Shader;

        type RenderCommand =
            WgpuRenderCommand<Self::Mesh, Self::Uniform, Self::ShaderHandle, Self::TextureHandle>;

        fn render(
            &mut self,
            render_commands: Vec<Self::RenderCommand>,
        ) -> Result<(), IndigoError<Self::ErrorMessage>> {
    
            //No idea whether the performance gain from ahash is significant but 
            //theres no downside to using it afaik
            let mut distinct_uniforms = Vec::<(Vec<u8>, wgpu::ShaderStages)>::new();
            let mut batches = AHashMap::<BatchInfo, Vec<Mesh>>::new();


            for command in render_commands {
                let Self::RenderCommand {
                    mesh,
                    shader,
                    textures,
                    uniforms,
                } = command;

                // Index into the distinct uniforms array
                let mut uniform_ids = Vec::new();
                
                //Map the uniforms into distinct uniform indices
                for uniform in uniforms {
                    match distinct_uniforms
                        .iter()
                        .enumerate()
                        .filter(|(_, stored_uni)| **stored_uni == uniform)
                        .next() {
                        
                            Some((index, _)) => uniform_ids.push(index),
                            None => {
                                distinct_uniforms.push(uniform);
                                uniform_ids.push(distinct_uniforms.len() - 1);
                            }
                            
                        }
                        
                        
                    }
                    
                let batch_info = BatchInfo::new(
                    &mesh,
                    shader,
                    textures,
                    uniform_ids
                ); 

                match batches.get_mut(&batch_info) {
                    Some(commands) => {
                        commands.push(mesh);
                    }
                    None => {
                        batches.insert(batch_info, vec![mesh]);
                    }
                };
            }

        
            //Filter out batches containing transparent meshes
            let transparent_batch_keys = batches
                .keys()
                .filter(|info| info.transparent)
                .cloned()
                .collect::<Vec<_>>();
            let mut transparent_batches = transparent_batch_keys
                .into_iter()
                .filter_map(|info| batches.remove_entry(&info))
                .collect::<Vec<_>>();

            //sort transparent batches back to front
            transparent_batches.sort_by(|(batch1, _), (batch2, _)| 
                batch1.highest_z.cmp(&batch2.highest_z)
            );
            
            // crate::debug!("Batches: {} ({} opaque + {} transparent)", transparent_batches.len() + batches.len(), batches.len(), transparent_batches.len());
            // for (info, batch) in &batches {
            //     crate::debug!("Opaque batch z={}: {} meshes", info.highest_z, batch.len());
            // }
            // for (info, batch) in &transparent_batches {
            //     crate::debug!("Transparent batch z={}: {} meshes", info.highest_z, batch.len());
            // }           
                
            //Render opaque meshes first and transparent ones second            
            let merged_batches = batches.into_iter().chain(transparent_batches).map(|(info, meshes)| {
                let mesh = meshes
                    .into_iter()
                    .reduce(|mut main, mut mesh| {
                        main.merge(&mut mesh);
                        main
                    })
                    .unwrap();

                (info, mesh)
            }).collect::<Vec<_>>();

            if let Err(wgpu::SurfaceError::Outdated) = self.context.render_batches(
                merged_batches,
                distinct_uniforms
            ) {
                return Err(IndigoError::FatalError { msg: "surface outdated".to_owned() });
            }

            Ok(())
        }

        fn on_window_resize(&mut self, new_window_size: (u32, u32)) {
            self.context.update_surface(new_window_size);
            self.main_camera.as_mut().unwrap().update(new_window_size);
            self.context.update_depth_texture(new_window_size);
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

        fn new_texture(
            &mut self, 
            data: &[u8], 
            dimensions: (u32, u32), 
            handle: Option<Self::TextureHandle>
        ) -> Self::TextureHandle {
            if handle.is_some() && self.context.textures.get(handle.unwrap()).is_some() {
                handle.unwrap()
            } else {
                let texture = Texture::new(
                    &self.context.device,
                    &self.context.queue,
                    data,
                    dimensions
                );

                self.context.textures.push(texture);
                let index = self.context.textures.len() - 1;

                index
            }
        }

        fn update_texture(
            &mut self, 
            texture_handle: Self::TextureHandle, 
            data: &[u8], 
        ) {
            if let Some(texture) = self.context.textures.get_mut(texture_handle) { 
                texture.update(&self.context.queue, data);
            }
        }

        fn load_texture(&mut self, texture_path: &std::path::Path) -> Self::TextureHandle {
            match self.texture_map.get(&texture_path.to_path_buf()) {
                Some(index) => *index,
                None => {

                    let image = image::open(texture_path).unwrap();
    
                    let texture = Texture::new(
                        &self.context.device, 
                        &self.context.queue, 
                        &image.to_rgba8(), 
                        image.dimensions()
                    ); 
    
                    self.context.textures.push(texture);
                    let index = self.context.textures.len() - 1;

                    self.texture_map.insert(texture_path.to_path_buf(), index);

                    index
                }
            }
        }

        fn remove_texture(&mut self, texture_handle: Self::TextureHandle) {
            self.context.textures.remove(texture_handle);
            
            let key = self.texture_map
                .iter()
                .find(|(key, value)| **value == texture_handle)
                .map(|(key, _)| key)
                .unwrap()
                .clone();

            self.texture_map.remove(&key);
        }

        fn load_shader(
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

        fn camera_uniform(&self) -> Self::Uniform {
            let camera_uniform = self.main_camera.as_ref().unwrap().uniform;
            let data = bytemuck::cast_slice(&[camera_uniform]).to_vec();
            (data, wgpu::ShaderStages::VERTEX)
        }
    }
}
