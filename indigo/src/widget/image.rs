
use std::path::PathBuf;

use rand::Rng;

use crate::graphics::IndigoRenderCommand;
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
    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            image_path: PathBuf::new(),
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
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
        _layout: Layout,
        _renderer: &mut R,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let plain_shader = crate::graphics::PLAIN_SHADER;
        let image_shader = crate::graphics::IMAGE_SHADER;
        let shader = _renderer.fetch_shader(plain_shader, "vs_main", image_shader, "fs_main");

        let mut mesh = DefaultMesh::<DefaultVertex>::quad((0.0, 0.0, 0.0), (500.0, 500.0));
        mesh.possibly_trasparent();
        let mesh = R::Mesh::convert(&mesh);
        
        let camera_uniform = _renderer.get_camera_uniform();

        let texture = _renderer.fetch_texture(&self.image_path);

        let mut command = R::RenderCommand::new(mesh, shader);
        command.add_uniform(camera_uniform);
        command.add_texture(texture);

        Ok(vec![command])
    }
}
