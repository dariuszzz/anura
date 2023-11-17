

use crate::context::RenderContext;
use crate::font::{Font, FontManager};
use crate::graphics::AnuraRenderCommand;
use crate::{
    app::App,
    context::AnuraContext,
    error::AnuraError,
    event::{WidgetEvent},
    graphics::AnuraRenderer,
    prelude::{
        DefaultMesh, DefaultVertex, FromAnuraMesh, FromAnuraUniform
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
    R: AnuraRenderer + 'static,    
    R::Mesh: FromAnuraMesh,
    R::Uniform: FromAnuraUniform,
    R::RenderCommand: AnuraRenderCommand<Renderer = R>
{

    fn handle_event(
        &mut self,
        ctx: &mut AnuraContext<'_, '_, A, V, R>,
        _view: &mut V,
        event: WidgetEvent,
    ) -> Result<(), AnuraError<R::ErrorMessage>> {
        match event {
            WidgetEvent::Init => {
                ctx.app.font_manager.load_font(&mut ctx.app.renderer, &self.font, false);
            },
            WidgetEvent::Update => {}
        };

        Ok(())
    }

    fn generate_mesh(
        &self,
        ctx: &mut RenderContext<'_, '_, A, V, R>,
        view: &mut V,
        layout: Layout,
    ) -> Result<Vec<R::RenderCommand>, AnuraError<R::ErrorMessage>>
    where     
    {
        let vert_code = crate::graphics::PLAIN_SHADER;
        let frag_code = crate::graphics::IMAGE_SHADER;

        let shader = ctx.app.renderer.load_shader(vert_code, "vs_main", frag_code, "fs_main");

        let font = ctx.app.font_manager.get_font(&self.font).expect("Font not loaded");

        let mut mesh = DefaultMesh::<DefaultVertex>::bounded_text(
            layout.origin, 
            Some(layout.available_space), 
            &self.text, 
            &font
        );
        mesh.possibly_trasparent();

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);

        let camera_uniform = ctx.app.renderer.camera_uniform();
        command.add_uniform(camera_uniform);
        command.add_texture(font.texture_handle.clone());

        Ok(vec![command])
    }
}