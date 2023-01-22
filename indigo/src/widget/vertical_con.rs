use crate::{app::App, uitree::UiTree, view::View, event::{IndigoResponse, WidgetEvent}, context::IndigoContext, graphics::IndigoRenderer, error::IndigoError, prelude::{FromIndigoMesh, FromIndigoUniform}};

use super::{Layout, Widget};

pub struct VerticalContainer {
}


impl<A, V, R> Widget<A, V, R> for VerticalContainer
where
    A: App<R>,
    V: View<A, R>,
    R: IndigoRenderer,
{
    fn default() -> Self
        where Self: Sized {
        Self {}
    }

    fn handle_event(
        &mut self, 
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
        event: WidgetEvent
    ) -> IndigoResponse {
        match event {
            WidgetEvent::Update => {
                
                // self.children // Vec<UntypedHandle>
                //     .iter() // Iterator<UntypedHandle>
                //     .with_context(_ctx) //????
                //     .for_each(|(ctx, widget)| {
                //         println!("siema")
                //     })
            },
            _ => {}
        };

        IndigoResponse::Noop
    }
    
    fn generate_mesh(&self, _layout: Layout, _renderer: &mut R) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        /*

        R::RenderCommand::new([0.0, 0.0, 0.0], _renderer.fetch_shader("main.spirv"))
            .uniform(...)
            .texture(_renderer.fetch_texture("asset.png"))

        */

        Ok(Vec::new())
    }
}