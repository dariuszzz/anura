

use crate::font::{Font, FontManager};
use crate::graphics::IndigoRenderCommand;
use crate::{
    app::App,
    context::IndigoContext,
    error::IndigoError,
    event::{WidgetEvent},
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
    A: App<R> + 'static,
    V: View<A, R> + 'static,
    R: IndigoRenderer + 'static,
{

    fn handle_event(
        &mut self,
        ctx: &mut IndigoContext<'_, '_, A, V, R>,
        _view: &mut V,
        event: WidgetEvent,
    ) -> Result<(), IndigoError<R::ErrorMessage>> {
        match event {
            WidgetEvent::Init => {
                ctx.app.font_manager.load_font(&mut ctx.app.renderer, &self.font, false);
            },
            WidgetEvent::Render { layout } => { 
                let commands = self.generate_mesh(&mut ctx.app.font_manager, &mut ctx.app.renderer, layout)?; 
                ctx.submit_render_commands(commands);
            },
            WidgetEvent::Update => {}
        };

        Ok(())
    }
}

impl TextWidget {
    pub fn generate_mesh<R>(
        &self,
        font_manager: &mut FontManager<R>,
        renderer: &mut R,
        layout: Layout,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>>
    where     
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
        let vert_code = crate::graphics::PLAIN_SHADER;
        let frag_code = crate::graphics::IMAGE_SHADER;

        let shader = renderer.load_shader(vert_code, "vs_main", frag_code, "fs_main");

        let font = font_manager.get_font(&self.font).expect("Font not loaded");

        let mut mesh = DefaultMesh::<DefaultVertex>::bounded_text(
            layout.origin, 
            Some(layout.available_space), 
            &self.text, 
            &font
        );
        mesh.possibly_trasparent();

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);

        let camera_uniform = renderer.camera_uniform();
        command.add_uniform(camera_uniform);
        command.add_texture(font.texture_handle.clone());

        Ok(vec![command])
    }
}