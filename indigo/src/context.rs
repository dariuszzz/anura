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
