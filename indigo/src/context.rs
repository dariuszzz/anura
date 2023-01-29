use crate::{uitree::UiTree, font::FontManager, graphics::IndigoRenderer};

pub struct MutIndigoContext<'a, A, V, O, R: IndigoRenderer> {
    pub app: &'a mut A,
    pub view: &'a mut O,
    pub ui_tree: &'a mut UiTree<A, V, R>,
    pub font_manager: &'a mut FontManager<R>,
    pub renderer: &'a mut R,
}

pub struct IndigoContext<'a, A, V, O, R: IndigoRenderer> {
    pub app: &'a A,
    pub view: &'a O,
    pub ui_tree: &'a UiTree<A, V, R>,
    pub font_manager: &'a FontManager<R>,
    pub renderer: &'a mut R,
}
