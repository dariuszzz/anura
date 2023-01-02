pub mod text;
use std::marker::PhantomData;

pub use text::*;

pub mod vertical_con;
pub use vertical_con::*;

use crate::{app::App, drawable, uitree::UiTree, view::View, event::{IndigoResponse, WidgetEvent}, context::IndigoContext, handle::{UntypedHandle}};

type IndigoRenderer = usize;

pub struct Layout {}

pub trait Widget<A, V>: std::any::Any
where
    A: App,
    V: View<A>,
{
    fn render(
        &mut self,
        _layout: Layout,
        _renderer: IndigoRenderer,
    ) -> Option<Box<dyn drawable::Drawable>> {
        None
    }

    fn handle_event<'a >(
        &mut self, 
        _ctx: &mut IndigoContext<'a, A, V, V>,
        _event: WidgetEvent
    ) -> IndigoResponse {
        IndigoResponse::Noop
    }

}

impl<A, V> dyn Widget<A, V> {
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














