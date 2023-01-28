
use std::path::PathBuf;

use crate::graphics::IndigoRenderCommand;
use crate::prelude::{IndigoUniform, IndigoShaderStage, MutIndigoContext};
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
pub struct Image {
    pub image_path: PathBuf,
}

impl<A, V, R> Widget<A, V, R> for Image
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
            WidgetEvent::Init { index: _ } => {},
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
        let plain_shader = crate::graphics::PLAIN_SHADER;
        let image_shader = crate::graphics::IMAGE_SHADER;
        let shader = renderer.load_shader(plain_shader, "vs_main", image_shader, "fs_main");

        let mut mesh = DefaultMesh::<DefaultVertex>::quad(
            layout.origin, 
            layout.available_space
        );
        mesh.possibly_trasparent();
        
        let camera_uniform = renderer.camera_uniform();

        let texture = renderer.load_texture(&self.image_path);

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);
        command.add_uniform(camera_uniform);
        command.add_texture(texture);

        Ok(vec![command])
    }
}