use crate::{app::App, drawable, uitree::UiTree, view::View, widget::Layout, context::IndigoContext, event::{IndigoResponse, WidgetEvent}, handle::{}};

use super::Widget;

pub struct TextWidget {
    pub text: String,
    pub index: Option<usize>,
}

better_any::tid!( impl<'a> TidAble<'a> for TextWidget);

impl<A, V> Widget<A, V> for TextWidget
where
    A: App,
    V: View<A>,
{
    // fn handle_indigo_events(
    //     &mut self,
    //     _app: &mut A,
    //     _view: &mut V,
    //     _ui_tree: &mut UiTree<A, V>,
    //     _event: IndigoEvent
    // ) {

    // }

    // fn handle_view_events(
    //     &mut self,
    //     _app: &mut A,
    //     _view: &mut V,
    //     _ui_tree: &mut UiTree<A, V>,
    //     _event: V::Event,
    // )


    fn handle_event(
        &mut self, 
        _ctx: &IndigoContext<'_, A, V, V>,
        event: WidgetEvent
    ) -> IndigoResponse {
        match event {
            WidgetEvent::Init { index } => self.index = Some(index),
            WidgetEvent::Update => {



            },//println!("{}", self.text), 
            _ => {}
        };

        IndigoResponse::Noop
    }

    fn render(&mut self, _layout: Layout, _renderer: usize) -> Option<Box<dyn drawable::Drawable>> {
        let text = drawable::Text {
            text: self.text.clone(),
            font: _renderer,
        };

        Some(Box::new(text))
    }
}
