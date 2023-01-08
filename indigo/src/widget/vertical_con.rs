use crate::{app::App, uitree::UiTree, view::View, event::{IndigoResponse, WidgetEvent}, context::IndigoContext, graphics::Renderer, error::IndigoError};

use super::{Layout, Widget};

pub struct VerticalContainer {
}


impl<A, V, R> Widget<A, V, R> for VerticalContainer
where
    A: App<R>,
    V: View<A, R>,
    R: Renderer
{
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
    
    fn render(&mut self, _layout: Layout, _renderer: &mut R) -> Result<(), IndigoError<R::ErrorMessage>> { Ok(()) }
}
