use crate::{uitree::UiTree, view::View, app::App, widget::Widget};

pub struct IndigoContext<'a, A, V, O>
where 
    A: App,
    V: View<A>,
{
    pub app: &'a mut A,
    pub view: &'a mut O,
    pub ui_tree: &'a mut UiTree<A, V>,
}



// pub(crate) struct IncompleteContext<'a, A: App> {
//     app: &'a mut A,
// }

// impl<'a, A: App> IncompleteContext<'a, A> {
//     pub(crate) fn new(app: &'a mut A) -> Self {
//         Self {
//             app
//         }
//     }

//     pub(crate) fn complete<V: View<A>>(self, view: &'a mut V, ui_tree: &'a mut UiTree<A, V>) -> IndigoContext<'a, A, V> {
//         IndigoContext {
//             app: self.app,
//             view: view,
//             ui_tree: ui_tree,
//         }
//     }
// }
