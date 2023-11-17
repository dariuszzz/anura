use std::error::Error;

use crate::{
    app::{App, AnuraApp},
    context::{AnuraContext, RenderContext},
    event::{ViewEvent, WidgetEvent},
    graphics::AnuraRenderer,
    prelude::{AnuraError, Layout, NodeType},
    uitree::UiTree, handle::UntypedHandle,
};

pub trait View<A, R>
where
    A: App<R>,
    R: AnuraRenderer,
    Self: Sized,
{
    fn handle_event(
        &mut self,
        _ctx: &mut AnuraContext<'_, '_, A, Self, R>,
        _event: ViewEvent,
    ) -> Result<(), AnuraError<R::ErrorMessage>>;
}

pub trait ViewWrapperTrait<A: App<R>, R: AnuraRenderer> {
    /// Updates the underlying View<A>
    fn update(&mut self, app: &mut AnuraApp<'_, A, R>) -> Result<(), AnuraError<R::ErrorMessage>>;
    fn render_view(
        &mut self,
        _window_size: (u32, u32),
        _app: &mut AnuraApp<'_, A, R>
    ) -> Result<Vec<R::RenderCommand>, AnuraError<R::ErrorMessage>>;
}

pub struct ViewWrapper<A, V, R> {
    view: V,
    ui_tree: UiTree<A, V, R>,
}

impl<'a, A, V, R> ViewWrapper<A, V, R>
where
    A: App<R>,
    V: View<A, R>,
    R: AnuraRenderer,
{
    pub fn new(mut view: V, app: &mut AnuraApp<'_, A, R>) -> ViewWrapper<A, V, R> {
        let mut ui_tree = UiTree::<A, V, R>::default();

        let mut context = AnuraContext { 
            app,
            ui_tree: &mut ui_tree,
            current: NodeType::Root,
        };

        view.handle_event(&mut context, ViewEvent::Init);

        Self { ui_tree, view }
    }
}

impl<A, V, R> ViewWrapperTrait<A, R> for ViewWrapper<A, V, R>
where
    A: App<R> + 'static,
    V: View<A, R> + 'static,
    R: AnuraRenderer + 'static,
{
    /// Inits all uninitialized widgets, updates them and then updates the underlying view
    fn update(&mut self, app: &mut AnuraApp<'_, A, R>) -> Result<(), AnuraError<R::ErrorMessage>> {
        // Not sure if this actually copies the pending_init vec but it definitely doesnt have to
        // maybe theres a better solution than .clone()?
        let mut pending_init = self.ui_tree.pending_init.clone();

        let mut context = AnuraContext { 
            app, 
            ui_tree: &mut self.ui_tree,
            current: NodeType::Root,
        };

        self.view.handle_event(&mut context, ViewEvent::Update)?;

        let handles = self.ui_tree.get_all_handles().collect::<Vec<_>>();

        for handle in &handles {
            //move the widget out to avoid aliasing refs
            let _: Result<(), AnuraError<R::ErrorMessage>> = self.ui_tree.run_on_moved_out(
                &handle, 
                |mut ui_tree, widget| {
                    let mut context = AnuraContext { 
                        app, 
                        ui_tree: &mut ui_tree,
                        current: NodeType::Handle(handle.clone())
                    };

                    if pending_init.contains(&handle) {
                        if let Err(err) = widget.handle_event(&mut context, &mut self.view, WidgetEvent::Init) {
                            return Err(err);
                        }

                        pending_init.retain(|h| *h != *handle);
                    }

                    widget.handle_event(&mut context, &mut self.view, WidgetEvent::Update)
                }
            );

        }

        self.ui_tree.pending_init = pending_init;

        Ok(())
    }

    fn render_view(
        &mut self,
        window_size: (u32, u32),
        app: &mut AnuraApp<'_, A, R>,
    ) -> Result<Vec<R::RenderCommand>, AnuraError<R::ErrorMessage>> {

        let mut command_vec = Vec::new();

        //Get handles of root children
        let handles = self.ui_tree.get_all_handles()
            .filter(|handle| match self.ui_tree.parent_arena.vec[handle.index].unwrap() {
                NodeType::Handle(_) => false,
                NodeType::Root => true
            })
            .collect::<Vec<_>>();

        for handle in handles {

            match app.render_cache.get(&handle) {
                None => {
                    let mut context = RenderContext {
                        app,
                        ui_tree: &self.ui_tree,
                        current: NodeType::Root
                    };
        
                    command_vec.append(&mut context.render(&handle, &mut self.view, Layout {
                        origin: (0.0, 0.0, 0.0),
                        available_space: (window_size.0 as f32, window_size.1 as f32)
                    })?);
                }
                Some(cmds) => {
                    command_vec.append(&mut cmds.clone());
                }
            }
        }


        Ok(command_vec)
    }
}
