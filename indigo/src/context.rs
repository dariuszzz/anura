use crate::uitree::UiTree;

pub struct MutIndigoContext<'a, A, V, O, R> {
    pub app: &'a mut A,
    pub view: &'a mut O,
    pub ui_tree: &'a mut UiTree<A, V, R>,
}

pub struct IndigoContext<'a, A, V, O, R> {
    pub app: &'a A,
    pub view: &'a O,
    pub ui_tree: &'a UiTree<A, V, R>,
}
