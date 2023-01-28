
use crate::graphics::IndigoRenderCommand;
use crate::prelude::MutIndigoContext;
use crate::{
    app::App,
    context::IndigoContext,
    error::IndigoError,
    event::{IndigoResponse, WidgetEvent},
    graphics::IndigoRenderer,
    prelude::{
        DefaultMesh, DefaultVertex, FromIndigoMesh, FromIndigoUniform
    },
    view::View,
    widget::Layout,
};

use super::Widget;

#[derive(Default)]
pub struct TextWidget {
    pub text: String,
    pub index: Option<usize>,
}

impl<A, V, R> Widget<A, V, R> for TextWidget
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

    fn handle_event(
        &mut self,
        _ctx: &mut MutIndigoContext<'_, A, V, V, R>,
        event: WidgetEvent,
    ) -> IndigoResponse {
        match event {
            WidgetEvent::Init { index } => self.index = Some(index),
            WidgetEvent::Update => {}
        };

        IndigoResponse::Noop
    }

    fn generate_mesh(
        &self,
        _ctx: &IndigoContext<'_, A, V, V, R>,
        layout: Layout,
        renderer: &mut R,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let shader_code = crate::graphics::PLAIN_SHADER;

        let shader = renderer.load_shader(shader_code, "vs_main", shader_code, "fs_main");

        let mut mesh = DefaultMesh::<DefaultVertex>::quad(
            layout.origin, 
            layout.available_space
        );
        mesh.possibly_trasparent();

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);

        let camera_uniform = renderer.camera_uniform();
        command.add_uniform(camera_uniform);

        Ok(vec![command])
    }
}
