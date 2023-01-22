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
    fn vertexLayout() -> Vec<VertexType>;
}

pub trait IndigoMesh {
    type Vertex: IndigoVertex;
    fn vertices(&self) -> Vec<Self::Vertex>;
    fn indices(&self) -> Vec<u16>;
}


pub trait IndigoUniform: bytemuck::Pod + bytemuck::Zeroable {

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
    
    type RenderCommand: IndigoRenderCommand<
        Mesh = Self::Mesh, 
        Uniform = Self::Uniform,
        TextureHandle = Self::TextureHandle, 
        ShaderHandle = Self::ShaderHandle 
    >;
    
    fn render(&self, render_commands: Vec<Self::RenderCommand>) -> Result<(), IndigoError<Self::ErrorMessage>>;

    fn setup_camera<V3f32: Into<[f32; 3]>>(
        &mut self,
        pos: V3f32,
        target: V3f32,
        up: V3f32,
        znear: f32,
        zfar: f32, 
    );

    fn on_window_resize(&mut self, new_window_size: impl Into<[u32; 2]>);

    fn fetch_texture(&mut self, texture_path: &Path) -> Self::TextureHandle;
    fn fetch_shader(&mut self, shader_path: &Path) -> Self::ShaderHandle;
    fn get_camera_uniform(&self) -> <Self::RenderCommand as IndigoRenderCommand>::Uniform;
}



#[cfg(feature = "wgpu-renderer")]
pub use wgpu_renderer_glue::*;

#[cfg(feature = "wgpu-renderer")]
mod wgpu_renderer_glue {

    pub use indigo_wgpu::*;
    use winit::window::Window;
    use crate::error::IndigoError;

    use super::{IndigoRenderer, FromIndigoMesh, IndigoRenderCommand, FromIndigoUniform};
    
    pub struct IndigoWgpuError(indigo_wgpu::wgpu::SurfaceError); 

    pub struct WgpuRenderer {
        context: renderer::RenderingContext,
        main_camera: Option<camera::Camera>,
    } 

    impl WgpuRenderer {
        pub async fn new(window: &Window) -> Self {
            let context = indigo_wgpu::renderer::RenderingContext::new(
                window.inner_size(), 
                window
            ).await;  

            Self {
                context, 
                main_camera: None,
            }
        }
    }

    pub struct TempMesh;

    impl FromIndigoMesh for TempMesh {
        fn convert<T: super::IndigoVertex>(indigo_mesh: &impl super::IndigoMesh<Vertex = T>) -> Self {
            Self {}
        }
    }

    pub struct TempUniformData;

    impl FromIndigoUniform for TempUniformData {
        fn convert(indigo_uniform: &impl super::IndigoUniform) -> Self {
            Self {}
        }
    }

        
    pub struct WgpuRenderCommand<M, U, S, T> {
        mesh: M,
        shader: S,
        textures: Vec<T>,
        uniforms: Vec<U>,
    }

    impl<M: FromIndigoMesh, U: FromIndigoUniform, S, T> IndigoRenderCommand for WgpuRenderCommand<M, U, S, T> {
        type Mesh = M;
        type Uniform = U;
        type ShaderHandle = S;
        type TextureHandle = T;

        fn new(mesh: Self::Mesh, shader: Self::ShaderHandle) -> Self {
            Self {
                mesh: mesh.into(),
                shader,
                textures: Vec::new(),
                uniforms: Vec::new(),
            }
        }

        fn add_texture(&mut self, texture: Self::TextureHandle) {
            self.textures.push(texture);
        }

        fn add_uniform(&mut self, uniform: Self::Uniform){
            self.uniforms.push(uniform.into());
        }
    }

    impl IndigoRenderer for WgpuRenderer {
        type ErrorMessage = String;
        type Mesh = TempMesh;
        type Uniform = TempUniformData;
        type TextureHandle = ();
        type ShaderHandle = ();
        
        type RenderCommand = WgpuRenderCommand<
            Self::Mesh, 
            Self::Uniform,
            Self::ShaderHandle,
            Self::TextureHandle,
        >;
        
        fn render(&self, render_commands: Vec<Self::RenderCommand>) -> Result<(), IndigoError<Self::ErrorMessage>> {
            let camera = self.main_camera.as_ref().unwrap();
            match self.context.temp_render(&camera.bind_group) {
                Err(wgpu::SurfaceError::Outdated) => Err(IndigoError::FatalError { msg: "surface outdated".to_owned() }),
                _ => Ok(())       
            }
            
        }

        fn on_window_resize(&mut self, new_window_size: impl Into<[u32; 2]>) {
            let [new_width, new_height] = new_window_size.into();
            self.context.update_surface((new_width, new_height));
        }

        fn setup_camera<V3f32: Into<[f32; 3]>>(
            &mut self,
            pos: V3f32,
            target: V3f32,
            up: V3f32,
            znear: f32,
            zfar: f32, 
        ) {
            //TODO: get rid of the into's
            self.main_camera = Some(self.context.new_camera(
                pos.into().into(), 
                target.into().into(),
                up.into().into(),
                znear,
                zfar
            ));
        }

        fn fetch_texture(&mut self, texture_path: &std::path::Path) -> Self::TextureHandle {
            unimplemented!()
        }

        fn fetch_shader(&mut self, shader_path: &std::path::Path) -> Self::ShaderHandle {
            todo!()
        }

        fn get_camera_uniform(&self) -> Self::Uniform {
            todo!()
        }

    }
}