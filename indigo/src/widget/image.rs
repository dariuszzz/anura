

use std::path::PathBuf;

use crate::context::RenderContext;
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
pub struct Image {
    pub image_path: PathBuf,
}

impl<A, V, R> Widget<A, V, R> for Image
where
    A: App<R> + 'static,
    V: View<A, R> + 'static,
    R: IndigoRenderer + 'static,
    R::Mesh: FromIndigoMesh,
    R::Uniform: FromIndigoUniform,
    R::RenderCommand: IndigoRenderCommand<Renderer = R>
{
    fn handle_event(
        &mut self,
        ctx: &mut IndigoContext<'_, '_, A, V, R>,
        _view: &mut V,
        event: WidgetEvent,
    ) -> Result<(), IndigoError<R::ErrorMessage>> {
        match event {
            WidgetEvent::Init => {},
            WidgetEvent::Update => {}
        };

        Ok(())
    }
    
    fn generate_mesh(
        &self,
        ctx: &mut RenderContext<'_, '_, A, V, R>,
        view: &mut V,
        layout: Layout,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let plain_shader = crate::graphics::PLAIN_SHADER;
        let image_shader = crate::graphics::IMAGE_SHADER;
        let shader = ctx.app.renderer.load_shader(plain_shader, "vs_main", image_shader, "fs_main");

        let mut mesh = DefaultMesh::<DefaultVertex>::quad(
            layout.origin, 
            layout.available_space,
            (0.0, 0.0, 1.0, 1.0),
            (0.0, 0.0, 0.0, 1.0)
        );
        mesh.possibly_trasparent();
        
        let camera_uniform = ctx.app.renderer.camera_uniform();

        let texture = ctx.app.renderer.load_texture(&self.image_path);

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);
        command.add_uniform(camera_uniform);
        command.add_texture(texture);

        Ok(vec![command])
    }
}