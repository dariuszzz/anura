use std::collections::VecDeque;

use crate::{uitree::UiTree, graphics::IndigoRenderer, app::{IndigoApp, App}, view::View, handle::{AsUntypedHandle, NodeType}, widget::Layout, event::WidgetEvent, error::IndigoError};

pub struct IndigoContext<'a, 'b, A, V, R>
where
    R: IndigoRenderer
{
    pub app: &'b mut IndigoApp<'a, A, R>,
    pub ui_tree: &'b mut UiTree<A, V, R>,
    pub current: NodeType,
}

impl<'a, 'b, A, V, R> IndigoContext<'a, 'b, A, V, R>
where
    A: App<R> + 'static,
    V: View<A, R> + 'static,
    R: IndigoRenderer + 'static,
{
    pub fn submit_render_commands(&mut self, commands: Vec<R::RenderCommand>) {
        if let NodeType::Handle(handle) = &self.current {
            self.app.render_cache.insert(handle.clone(), commands);
        } else {
            panic!("Cannot submit commands for a non widget")
        }
    }

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

    pub fn render(
        &mut self, 
        handle: &impl AsUntypedHandle, 
        view: &mut V,
        layout: Layout
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        let handle = handle.handle();
        
        if let NodeType::Handle(curr_handle) = &self.current {
            if *curr_handle == handle {
                panic!("Cannot render self")
            } 
        }

        if self.app.render_cache.get(&handle).is_none() {
            self.ui_tree.run_on_moved_out(&handle, |ui_tree, widget| {
                let mut ctx = IndigoContext { 
                    app: self.app,
                    ui_tree,
                    current: NodeType::Handle(handle)
                };
    
                widget.handle_event(&mut ctx, view, WidgetEvent::Render { layout })
            })?;
        }

        Ok(self.app.render_cache.get(&handle.handle())
            .expect("Render event didnt submit a command vec")
            .clone())



    }
}