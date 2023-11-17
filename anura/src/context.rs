use std::collections::VecDeque;

use crate::{uitree::UiTree, graphics::AnuraRenderer, app::{AnuraApp, App}, view::View, handle::{AsUntypedHandle, NodeType}, widget::Layout, event::WidgetEvent, error::AnuraError};

pub struct AnuraContext<'a, 'b, A, V, R>
where
    R: AnuraRenderer
{
    pub app: &'b mut AnuraApp<'a, A, R>,
    pub ui_tree: &'b mut UiTree<A, V, R>,
    pub current: NodeType,
}

impl<'a, 'b, A, V, R> AnuraContext<'a, 'b, A, V, R>
where
    A: App<R> + 'static,
    V: View<A, R> + 'static,
    R: AnuraRenderer + 'static,
{
    pub fn issue_rerender(&mut self) {
        if let NodeType::Handle(handle) = &self.current {

            let mut handles = VecDeque::new();
            handles.push_back(handle.clone());
            loop {
                let curr_handle = match handles.pop_front() {
                    Some(handle) => handle,
                    None => break, 
                };

                handles.extend(self.ui_tree.get_children_handles(&curr_handle));
                self.app.render_cache.remove(&curr_handle);
            }

        } else { 
            panic!("Cannot rerender a non widget")
        }
    }
}

pub struct RenderContext<'a, 'b, A, V, R>
where
    R: AnuraRenderer
{
    pub app: &'b mut AnuraApp<'a, A, R>,
    pub ui_tree: &'b UiTree<A, V, R>,
    pub current: NodeType,
}


impl<'a, 'b, A, V, R> RenderContext<'a, 'b, A, V, R>
where
    A: App<R> + 'static,
    V: View<A, R> + 'static,
    R: AnuraRenderer + 'static,
{
    
    pub fn render(
        &mut self, 
        handle: &impl AsUntypedHandle, 
        view: &mut V,
        layout: Layout
    ) -> Result<Vec<R::RenderCommand>, AnuraError<R::ErrorMessage>> {
        let handle = handle.handle();
        
        if let NodeType::Handle(curr_handle) = &self.current {
            if *curr_handle == handle {
                panic!("Recursive generate_mesh")
            } 
        }

        if self.app.render_cache.get(&handle).is_none() {
            let widget = self.ui_tree.get_untyped_ref(&handle).unwrap();
            let mut ctx = RenderContext { 
                app: self.app,
                ui_tree: self.ui_tree,
                current: NodeType::Handle(handle)
            };

            let commands = widget.generate_mesh(&mut ctx, view, layout)?;
            self.app.render_cache.insert(handle.clone(), commands);
        }

        Ok(self.app.render_cache.get(&handle.handle())
            .expect("Render event didnt submit a command vec")
            .clone())
    }
}