use std::path::Path;

use crate::error::IndigoError;


pub trait RenderCommand {

    type Mesh;
    type Shader;
    type Texture;
    type Uniform;

    fn new(mesh: impl Into<Self::Mesh>, shader_path: Self::Shader) -> Self;
    fn add_uniform(&mut self, uniform: impl Into<Self::Uniform>);
    fn add_texture(&mut self, texture: Self::Texture);
}

pub struct DefaultRenderCommand<M, S, T, U> {
    mesh: M,
    shader: S,
    textures: Vec<T>,
    uniforms: Vec<U>,
}

impl<M, S, T, U> RenderCommand for DefaultRenderCommand<M, S, T, U> {
    type Mesh = M;
    type Shader = S;
    type Texture = T;
    type Uniform = U;

    fn new(mesh: impl Into<Self::Mesh>, shader: Self::Shader) -> Self {
        Self {
            mesh: mesh.into(),
            shader,
            textures: Vec::new(),
            uniforms: Vec::new(),
        }
    }

    fn add_texture(&mut self, texture: Self::Texture) {
        self.textures.push(texture);
    }

    fn add_uniform(&mut self, uniform: impl Into<Self::Uniform>){
        self.uniforms.push(uniform.into());
    }
}


pub trait Renderer {
    type ErrorMessage: std::fmt::Debug;
    type RenderCommand: RenderCommand;
    type TextureHandle;
    type ShaderHandle;

    fn render(&self, render_commands: Vec<Self::RenderCommand>) -> Result<(), IndigoError<Self::ErrorMessage>>;

    fn setup_camera(
        &mut self,
        pos: impl Into<[f32; 3]>,
        target: impl Into<[f32; 3]>,
        up: impl Into<[f32; 3]>,
        znear: f32,
        zfar: f32, 
    );

    fn on_window_resize(&mut self, new_window_size: impl Into<[u32; 2]>);

    fn fetch_texture(&mut self, texture_path: &Path) -> Self::TextureHandle;
    fn fetch_shader(&mut self, shader_path: &Path) -> Self::ShaderHandle;
}

#[cfg(feature = "wgpu-renderer")]
pub use wgpu_renderer_glue::*;

#[cfg(feature = "wgpu-renderer")]
mod wgpu_renderer_glue {

    pub use indigo_wgpu::*;
    use winit::window::Window;
    use crate::error::IndigoError;

    use super::{Renderer, DefaultRenderCommand};
    
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

    impl Renderer for WgpuRenderer {
        type ErrorMessage = String;
        type RenderCommand = DefaultRenderCommand<
            (), 
            (), // Something like RenderingContext::ShaderHandle
            Self::TextureHandle,
            Vec<wgpu::BindGroupLayoutEntry>,
        >;
        type TextureHandle = ();
        type ShaderHandle = ();

        fn render(&self, render_commands: Vec<Self::RenderCommand>) -> Result<(), IndigoError<Self::ErrorMessage>> {
            let camera = self.main_camera.as_ref().unwrap();
            match self.context.temp_render(&camera.bind_group) {
                Err(wgpu::SurfaceError::Outdated) => Err(IndigoError::FatalError { msg: "surface outdated".to_owned() }),
                _ => Ok(())       
            }
            
        }

        fn on_window_resize(&mut self, new_window_size: impl Into<[u32; 2]>) {
            // todo!()
        }

        fn setup_camera(
            &mut self,
            pos: impl Into<[f32; 3]>,
            target: impl Into<[f32; 3]>,
            up: impl Into<[f32; 3]>,
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
            todo!()
        }

        fn fetch_shader(&mut self, shader_path: &std::path::Path) -> Self::ShaderHandle {
            todo!()
        }

    }
}