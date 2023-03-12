


use crate::{
    app::App,
    context::{IndigoContext, RenderContext},
    error::IndigoError,
    event::{WidgetEvent},
    graphics::IndigoRenderer,
    prelude::{DefaultMesh, DefaultVertex, FromIndigoMesh, FromIndigoUniform, IndigoRenderCommand, UntypedHandle, AsUntypedHandle},
    view::View, handle::NodeType,
};

use super::{Layout, Widget};

#[derive(Default)]
pub struct VerticalContainer {
    pub gap: f32,
    pub children: Vec<UntypedHandle>,
}

impl VerticalContainer {
    pub fn add_child(&mut self, child_handle: impl AsUntypedHandle) {
        self.children.push(child_handle.handle());
    }
}

impl<A, V, R> Widget<A, V, R> for VerticalContainer
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
        view: &mut V,
        event: WidgetEvent,
    ) -> Result<(), IndigoError<R::ErrorMessage>> {
        match event {
            WidgetEvent::Update => {}
            _ => {}
        };

        Ok(())
    }
    
    fn generate_mesh(
        &self,
        ctx: &mut RenderContext<'_, '_, A, V, R>,
        view: &mut V,
        layout: Layout,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let Layout {
            origin,
            available_space
        } = layout;

        let mut commands = Vec::new();

        let max_y_per_child = available_space.1 / self.children.len() as f32;

        for (i, child) in self.children
            .iter()
            .enumerate() 
        {
            let mut child_commands = ctx.render(
                child,
                view,
                Layout {
                    origin: (origin.0, origin.1 + i as f32 * max_y_per_child, origin.2 + 0.1),
                    available_space: (available_space.0, max_y_per_child)
                }, 
            )?;

            commands.append(&mut child_commands);
        } 

        let shader_code = crate::graphics::PLAIN_SHADER;

        let shader = ctx.app.renderer.load_shader(shader_code, "vs_main", shader_code, "fs_main");

        let mesh = DefaultMesh::<DefaultVertex>::quad(
            origin, 
            available_space,
            (0.0,0.0,0.0,0.0),
            (0.4, 0.2, 0.3, 1.0)
        );

        let mut command = R::RenderCommand::new(R::Mesh::convert(&mesh), shader);

        let camera_uniform = ctx.app.renderer.camera_uniform();
        command.add_uniform(camera_uniform);
        
        commands.push(command);

        Ok(commands)
    }
}
