use std::{path::Path};

mod default_impls;
pub use default_impls::*;

use crate::error::AnuraError;

//For some reason this doesnt compile when Anura_mesh is &T where T: AnuraMesh<Vertex = V>
//so it will stay as &impl AnuraMesh<Vertex = V> for now 
pub trait FromAnuraMesh {
    fn convert<V: AnuraVertex>(Anura_mesh: &impl AnuraMesh<Vertex = V>) -> Self;
}

pub trait FromAnuraUniform {
    fn convert<T: AnuraUniform>(Anura_uniform: &T) -> Self;
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

pub trait AnuraVertex: bytemuck::Pod + bytemuck::Zeroable {
    fn vertex_layout() -> Vec<VertexType>;
}

pub trait AnuraMesh {
    type Vertex: AnuraVertex;
    fn vertices(&self) -> Vec<Self::Vertex>;
    fn indices(&self) -> Vec<u16>;
    fn could_be_transparent(&self) -> bool;
    fn highest_z(&self) -> f32; 
}

pub enum AnuraShaderStage {
    Both,
    Vertex,
    Fragment,
}

pub trait AnuraUniform: bytemuck::Pod + bytemuck::Zeroable {
    const SHADER_STAGE: AnuraShaderStage; 
}

pub trait AnuraRenderCommand: Clone {
    type Renderer: AnuraRenderer;

    fn new(mesh: <Self::Renderer as AnuraRenderer>::Mesh, shader: <Self::Renderer as AnuraRenderer>::ShaderHandle) -> Self;
    fn add_uniform(&mut self, uniform: <Self::Renderer as AnuraRenderer>::Uniform);
    fn add_texture(&mut self, texture: <Self::Renderer as AnuraRenderer>::TextureHandle);
}

pub trait AnuraRenderer {
    type ErrorMessage: std::fmt::Debug + std::fmt::Display;

    //Constrain mesh and uniform for the default renderer so custom views/apps/widgets
    //dont have to specify a concrete renderer type/dont have to add these constraints themselves
    #[cfg(feature = "wgpu-renderer")]
    type Mesh: FromAnuraMesh;
    #[cfg(feature = "wgpu-renderer")]
    type Uniform: FromAnuraUniform;

    //If the default renderer is not being used then these can be unconstrained in case
    //default widgets are not desired (they still can use them if their mesh/uniform types
    //implement FromAnuraMesh and FromAnuraUniform respectively)
    #[cfg(not(feature = "wgpu-renderer"))]
    type Mesh;
    #[cfg(not(feature = "wgpu-renderer"))]
    type Uniform;

    type TextureHandle: Clone;
    type ShaderHandle;

    // Only force AnuraRenderCommand constraint for the default renderer
    // a custom renderer's rendercommand would have to implement it anyway in order to
    #[cfg(feature = "wgpu-renderer")]
    type RenderCommand: AnuraRenderCommand<
        Renderer = Self
    >;

    #[cfg(not(feature = "wgpu-renderer"))]
    type RenderCommand;

    fn render(
        &mut self,
        render_commands: Vec<Self::RenderCommand>,
    ) -> Result<(), AnuraError<Self::ErrorMessage>>;

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

    use crate::{error::AnuraError};
    use ahash::AHashMap;
    use image::GenericImageView;
    pub use wgduck::*;
    use wgduck::{
        camera::{OrthoCameraa, OrthoCamera},
        mesh::{VertexLayoutInfo, Mesh, PackedMesh},
        shader::Shader,
        wgpu::VertexFormat, renderer::BatchInfo, texture::Texture,
    };
    use winit::window::Window;

    use super::{FromAnuraMesh, FromAnuraUniform, AnuraRenderCommand, AnuraRenderer, AnuraShaderStage};

    pub struct AnuraWgpuError(wgduck::wgpu::SurfaceError);

    pub struct WgpuRenderer {
        context: renderer::RenderingContext,
        texture_map: AHashMap<PathBuf, usize>,
        main_camera: Option<camera::OrthoCamera>,
    }

    impl WgpuRenderer {
        pub async fn new(window: &Window) -> Self {
            let context =
                wgduck::renderer::RenderingContext::new(window.inner_size(), window).await;

            Self {
                context,
                texture_map: AHashMap::new(),
                main_camera: None,
            }
        }
    }

    impl FromAnuraMesh for PackedMesh {
        fn convert<V: super::AnuraVertex>(
            Anura_mesh: &impl super::AnuraMesh<Vertex = V>,
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
                vertices: bytemuck::cast_slice(Anura_mesh.vertices().as_slice()).to_vec(),
                indices: Anura_mesh.indices(),
                layout: VertexLayoutInfo {
                    array_stride: std::mem::size_of::<V>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes,
                },
                could_be_transparent: Anura_mesh.could_be_transparent(),
                highest_z: Anura_mesh.highest_z()
            }
        }
    }

    impl FromAnuraUniform for (Vec<u8>, wgpu::ShaderStages) {
        fn convert<T: super::AnuraUniform>(Anura_uniform: &T) -> Self {
            //hardcoded shader stage for now
            let shader_stage = match T::SHADER_STAGE {
                AnuraShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
                AnuraShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
                AnuraShaderStage::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
            };

            (
                bytemuck::cast_slice(&[*Anura_uniform]).to_vec(),
                shader_stage,
            )
        }
    }

    #[derive(Clone)]
    pub struct WgpuRenderCommand {
        pub mesh: <WgpuRenderer as AnuraRenderer>::Mesh,
        pub shader: <WgpuRenderer as AnuraRenderer>::ShaderHandle,
        pub textures: Vec<<WgpuRenderer as AnuraRenderer>::TextureHandle>,
        pub uniforms: Vec<<WgpuRenderer as AnuraRenderer>::Uniform>,
    }

    impl AnuraRenderCommand for WgpuRenderCommand {
        type Renderer = WgpuRenderer;

        fn new(mesh: <WgpuRenderer as AnuraRenderer>::Mesh, shader: <WgpuRenderer as AnuraRenderer>::ShaderHandle) -> Self {
            Self {
                mesh,
                shader,
                textures: Vec::new(),
                uniforms: Vec::new(),
            }
        }

        fn add_texture(&mut self, texture: <WgpuRenderer as AnuraRenderer>::TextureHandle) {
            self.textures.push(texture);
        }

        fn add_uniform(&mut self, uniform: <WgpuRenderer as AnuraRenderer>::Uniform) {
            self.uniforms.push(uniform);
        }
    }

    impl AnuraRenderer for WgpuRenderer {
        type ErrorMessage = String;
        type Mesh = PackedMesh;
        type Uniform = (Vec<u8>, wgpu::ShaderStages);
        type TextureHandle = wgduck::renderer::TextureHandle;
        type ShaderHandle = Shader;

        type RenderCommand = WgpuRenderCommand;

        fn render(
            &mut self,
            render_commands: Vec<Self::RenderCommand>,
        ) -> Result<(), AnuraError<Self::ErrorMessage>> {
    
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

                if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                    continue;
                }

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
                return Err(AnuraError::FatalError { msg: "surface outdated".to_owned() });
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
            let mut camera = OrthoCamera::new(pos.into(), target.into(), up.into(), znear, zfar, 1.0);

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
                .find(|(_, value)| **value == texture_handle)
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
