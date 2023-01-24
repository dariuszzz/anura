use crate::uitree::UiTree;

pub struct IndigoContext<'a, A, V, O, R> {
    pub app: &'a mut A,
    pub view: &'a mut O,
    pub ui_tree: &'a mut UiTree<A, V, R>,
}
