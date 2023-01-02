use crate::{app::App, drawable, uitree::UiTree, view::View, event::{IndigoResponse, WidgetEvent}, context::IndigoContext};

use super::{Layout, Widget};

pub struct VerticalContainer {
    pub color: drawable::Color,
}


impl<A, V> Widget<A, V> for VerticalContainer
where
    A: App,
    V: View<A>
{
    fn handle_event(
        &mut self, 
        _ctx: &mut IndigoContext<'_, A, V, V>,
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
    
    fn render(&mut self, _layout: Layout, _renderer: usize) -> Option<Box<dyn drawable::Drawable>> {
        let bg = drawable::Quad {};

        Some(Box::new(bg))
    }
}
