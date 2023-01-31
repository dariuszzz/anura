use crate::{uitree::UiTree, graphics::IndigoRenderer, app::IndigoApp};

pub struct IndigoContext<'a, 'b, A, V, R>
where
    R: IndigoRenderer
{
    pub app: &'b mut IndigoApp<'a, A, R>,
    pub ui_tree: &'b mut UiTree<A, V, R>,
}