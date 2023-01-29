
use std::default;
use std::path::{PathBuf, Path};

use ordered_float::NotNan;

use crate::font::Font;
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
    pub font: Font,
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
            WidgetEvent::Init { index } => {
                _ctx.font_manager.load_font(_ctx.renderer, &self.font, false);

                self.index = Some(index)
            },
            WidgetEvent::Update => {}
        };

        IndigoResponse::Noop
    }

    fn generate_mesh(
        &self,
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
        layout: Layout,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let vert_code = crate::graphics::PLAIN_SHADER;
        let frag_code = crate::graphics::IMAGE_SHADER;

        let shader = _ctx.renderer.load_shader(vert_code, "vs_main", frag_code, "fs_main");

        let font = _ctx.font_manager.get_font(&self.font).expect("Font not loaded");

        let mut mesh = DefaultMesh::<DefaultVertex>::bounded_text(
            layout.origin, 
            Some(layout.available_space), 
            &self.text, 
            &font
        );
        mesh.possibly_trasparent();

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);

        let camera_uniform = _ctx.renderer.camera_uniform();
        command.add_uniform(camera_uniform);
        command.add_texture(font.texture_handle.clone());

        Ok(vec![command])
    }
}
