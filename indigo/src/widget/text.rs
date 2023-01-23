use std::path::Path;

use indigo_wgpu::wgpu::Texture;
use rand::Rng;
use relative_path::RelativePath;

use crate::{app::App, uitree::UiTree, view::View, widget::Layout, context::IndigoContext, event::{IndigoResponse, WidgetEvent}, handle::{}, graphics::IndigoRenderer, error::IndigoError, prelude::{IndigoMesh, IndigoUniform, FromIndigoMesh, FromIndigoUniform, DefaultMesh, DefaultVertex}};
use crate::graphics::IndigoRenderCommand;

use super::Widget;

pub struct TextWidget {
    pub text: String,
    pub index: Option<usize>,
}

impl<'a, A, V, R> Widget<A, V, R> for TextWidget
where
    A: App<R>,
    V: View<A, R>,
    R: IndigoRenderer,
    R::Mesh: FromIndigoMesh,
    R::Uniform: FromIndigoUniform,
    R::RenderCommand: IndigoRenderCommand<
        Mesh = R::Mesh,
        Uniform = R::Uniform,
        ShaderHandle = R::ShaderHandle,
        TextureHandle = R::TextureHandle
    >
{
    fn default() -> Self
        where Self: Sized {
        Self { text: String::new(), index: None }
    }

    fn handle_event(
        &mut self, 
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
        event: WidgetEvent
    ) -> IndigoResponse {
        match event {
            WidgetEvent::Init { index } => self.index = Some(index),
            WidgetEvent::Update => {},
            _ => {}
        };

        IndigoResponse::Noop
    }

    fn generate_mesh(&self, _layout: Layout, _renderer: &mut R) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> { 
        
        let shader_code = crate::graphics::DEFAULT_SHADER;

        let shader = _renderer.fetch_shader(
            shader_code,
            "vs_main",
            shader_code,
            "fs_main"
        );

        //Temporary, remove later
        let mut rng = rand::thread_rng();

        let mesh = DefaultMesh::<DefaultVertex>::quad(
            rng.gen_range(0.0..500.0), rng.gen_range(0.0..500.0), 
            100.0, 100.0
        );
        let mesh = R::Mesh::convert(&mesh);

        let mut command = R::RenderCommand::new(mesh, shader);

        let camera_uniform = _renderer.get_camera_uniform();
        command.add_uniform(camera_uniform);

        Ok(vec![command]) 
    }
}