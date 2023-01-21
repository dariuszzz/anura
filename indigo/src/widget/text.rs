use crate::{app::App, uitree::UiTree, view::View, widget::Layout, context::IndigoContext, event::{IndigoResponse, WidgetEvent}, handle::{}, graphics::Renderer, error::IndigoError};

use super::Widget;

pub struct TextWidget {
    pub text: String,
    pub index: Option<usize>,
}

// better_any::tid!( impl<'a> TidAble<'a> for TextWidget);

impl<A, V, R> Widget<A, V, R> for TextWidget
where
    A: App<R>,
    V: View<A, R>,
    R: Renderer,
{
    fn default() -> Self
        where Self: Sized {
        Self { text: String::new(), index: None }
    }

    fn handle_event(
        &mut self, 
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
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

    fn generate_mesh(&mut self, _layout: Layout, _renderer: &mut R) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> { Ok(Vec::new()) }
}
