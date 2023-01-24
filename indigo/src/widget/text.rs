
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

pub struct TextWidget {
    pub text: String,
    pub index: Option<usize>,
    pub x_pos: f32, //Temp
    pub y_pos: f32, //
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
    fn default() -> Self
    where
        Self: Sized,
    {
        let mut rng = rand::thread_rng();

        let x_pos = rng.gen_range(0.0..500.0);
        let y_pos = rng.gen_range(0.0..500.0);

        Self {
            text: String::new(),
            index: None,
            x_pos,
            y_pos,
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
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
        _layout: Layout,
        _renderer: &mut R,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let shader_code = crate::graphics::DEFAULT_SHADER;

        let shader = _renderer.fetch_shader(shader_code, "vs_main", shader_code, "fs_main");

        let mesh = DefaultMesh::<DefaultVertex>::quad(self.x_pos, self.y_pos, 100.0, 100.0);
        let mesh = R::Mesh::convert(&mesh);

        let mut command = R::RenderCommand::new(mesh, shader);

        let camera_uniform = _renderer.get_camera_uniform();
        command.add_uniform(camera_uniform);

        Ok(vec![command])
    }
}
