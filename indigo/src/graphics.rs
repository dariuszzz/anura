use crate::error::IndigoError;

pub trait Renderer {
    type ErrorMessage: std::fmt::Debug;

    fn render(&self) -> Result<(), IndigoError<Self::ErrorMessage>>;

    fn setup_camera(
        &mut self,
        pos: impl Into<[f32; 3]>,
        target: impl Into<[f32; 3]>,
        up: impl Into<[f32; 3]>,
        znear: f32,
        zfar: f32, 
    );

    fn on_window_resize(&mut self, new_window_size: impl Into<[u32; 2]>);
}

#[cfg(feature = "wgpu-renderer")]
pub use wgpu_renderer_glue::*;

#[cfg(feature = "wgpu-renderer")]
mod wgpu_renderer_glue {
    use indigo_wgpu::*;
    use winit::window::{self, Window};
    use crate::error::IndigoError;

    use super::Renderer;
    
    pub struct IndigoWgpuError(indigo_wgpu::wgpu::SurfaceError); 

    pub struct WgpuRenderer {
        context: renderer::RenderingContext,
        main_camera: Option<camera::Camera>,
    } 

    impl WgpuRenderer {
        pub async fn new(window: &Window) -> Self {
            let context = indigo_wgpu::renderer::RenderingContext::new(
                window.inner_size(), 
                &window
            ).await;  

            Self {
                context, 
                main_camera: None,
            }
        }
    }

    impl Renderer for WgpuRenderer {
        type ErrorMessage = String;

        fn render(&self) -> Result<(), IndigoError<Self::ErrorMessage>> {
            let camera = self.main_camera.as_ref().unwrap();
            match self.context.temp_render(&camera.bind_group) {
                Err(wgpu::SurfaceError::Outdated) => Err(IndigoError::FatalError { msg: "surface outdated".to_owned() }),
                _ => Ok(())       
            }
            
        }

        fn on_window_resize(&mut self, new_window_size: impl Into<[u32; 2]>) {
            
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

    }
}