pub mod text;
use std::marker::PhantomData;

pub use text::*;

pub mod vertical_con;
pub use vertical_con::*;

use crate::{app::App, drawable, uitree::UiTree, view::View, event::{IndigoResponse, WidgetEvent}, context::IndigoContext, handle::{UntypedHandle}};

type IndigoRenderer = usize;

pub struct Layout {}




// pub trait WidgetWrapperTrait<A, V>
// where 
//     A: App,
//     V: View<A>
// {
//     fn send_event(&mut self, _ctx: &mut IndigoContext<A, V>, _event: WidgetEvent) -> IndigoResponse;

//     fn get_widget_ref(&self) -> &dyn Widget<A, V>;
//     fn get_widget_mut(&mut self) -> &mut dyn Widget<A, V>;
//     fn get_children(&self) -> Vec<UntypedHandle>;
//     fn get_parent(&self) -> Option<UntypedHandle>;

// }

// pub struct WidgetWrapper<A, V>
// where
//     A: App,
//     V: View<A>,
// {
//     pub(crate) widget: ,
//     pub(crate) children: Vec<UntypedHandle>, 
//     pub(crate) parent: Option<UntypedHandle>,
// }

// impl<A, V> WidgetWrapper<A, V>
// where
//     A: App + 'static,
//     V: View<A> + 'static,
// {

//     pub fn new(widget: Box<dyn Widget<A, V>>, parent: Option<impl Into<UntypedHandle>>) -> Self {
//         Self {
//             widget,
//             children: Vec::new(),
//             parent: parent.and_then(|h| Some(h.into()))
//         }
//     }

//     pub fn send_event(&mut self, ctx: &mut IndigoContext<A, V>, _event: WidgetEvent) -> IndigoResponse {
//         self.widget.handle_event(ctx, _event)
//     }

//     // pub fn get_widget_ref(&self) -> &dyn Widget<A, V> { &self.widget }
//     // pub fn get_widget_mut(&mut self) -> &mut dyn Widget<A, V> { &mut self.widget }
//     // pub fn get_children(&self) -> Vec<UntypedHandle> { self.children.clone() }
//     // pub fn get_parent(&self) -> Option<UntypedHandle> { self.parent.clone() }
// }



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
        _ctx: &IndigoContext<'a, A, V, V>,
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














