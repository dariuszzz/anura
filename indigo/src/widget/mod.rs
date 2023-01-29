pub mod text;
pub use text::*;

pub mod vertical_con;
pub use vertical_con::*;

pub mod image;
pub use crate::widget::image::*;

use crate::{
    app::App,
    context::IndigoContext,
    error::IndigoError,
    event::{IndigoResponse, WidgetEvent},
    graphics::IndigoRenderer,
    view::View, prelude::MutIndigoContext,
};

pub struct Layout {
    pub origin: (f32, f32, f32),
    pub available_space: (f32, f32),
}

pub trait Widget<A, V, R>: std::any::Any
where
    A: App<R>,
    V: View<A, R>,
    R: IndigoRenderer,
{
    fn handle_event(
        &mut self,
        _ctx: &mut MutIndigoContext<'_, A, V, V, R>,
        _event: WidgetEvent,
    ) -> IndigoResponse {
        IndigoResponse::Noop
    }

    fn generate_mesh(
        &self,
        _ctx: &mut IndigoContext<'_, A, V, V, R>,
        _layout: Layout,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        Ok(Vec::new())
    }
}