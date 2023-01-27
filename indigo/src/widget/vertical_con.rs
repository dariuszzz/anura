
use crate::{
    app::App,
    context::IndigoContext,
    error::IndigoError,
    event::{IndigoResponse, WidgetEvent},
    graphics::IndigoRenderer,
    prelude::{DefaultMesh, DefaultVertex, FromIndigoMesh, FromIndigoUniform, IndigoRenderCommand},
    view::View,
};

use super::{Layout, Widget};

pub struct VerticalContainer {}

impl<A, V, R> Widget<A, V, R> for VerticalContainer
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
        TextureHandle = R::TextureHandle,
    >,
{
    fn default() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn handle_event(
        &mut self,
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
        event: WidgetEvent,
    ) -> IndigoResponse {
        match event {
            WidgetEvent::Update => {}
            _ => {}
        };

        IndigoResponse::Noop
    }

    fn generate_mesh(
        &self,
        _layout: Layout,
        _renderer: &mut R,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let shader_code = crate::graphics::PLAIN_SHADER;

        let shader = _renderer.fetch_shader(shader_code, "vs_main", shader_code, "fs_main");

        let mesh = DefaultMesh::<DefaultVertex>::quad((500.0, 500.0, 1.0), (100.0, 100.0));

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);

        let camera_uniform = _renderer.get_camera_uniform();
        command.add_uniform(camera_uniform);
        
        Ok(vec![command])
    }
}
