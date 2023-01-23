pub mod text;
pub use text::*;

pub mod vertical_con;
pub use vertical_con::*;

use crate::{app::App, uitree::UiTree, view::View, event::{IndigoResponse, WidgetEvent}, context::IndigoContext, handle::{UntypedHandle}, graphics::IndigoRenderer, error::IndigoError, prelude::IndigoRenderCommand};


pub struct Layout {}



pub trait Widget<A, V, R>: std::any::Any
where
    A: App<R>,
    V: View<A, R>,
    R: IndigoRenderer,
{
    // Custom default method since Default doesnt constrain Self to Sized which is required
    // for object safety
    fn default() -> Self
    where Self: Sized;

    fn handle_event<'a >(
        &mut self, 
        _ctx: &mut IndigoContext<'a, A, V, V, R>,
        _event: WidgetEvent
    ) -> IndigoResponse {
        IndigoResponse::Noop
    }

    fn generate_mesh(
        &self,
        _layout: Layout,
        _renderer: &mut R,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        Ok(Vec::new())
    }
} 



//i dont like this
impl<A, V, R> dyn Widget<A, V, R>
{
    pub fn as_any_ref(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    pub fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.as_any_ref().is::<T>()
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any_ref().downcast_ref::<T>()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}












