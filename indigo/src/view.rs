use std::error::Error;

use crate::{
    app::{App, IndigoApp},
    context::IndigoContext,
    event::{ViewEvent, WidgetEvent},
    graphics::IndigoRenderer,
    prelude::{IndigoError, Layout, NodeType},
    uitree::UiTree, handle::UntypedHandle,
};

pub trait View<A, R>
where
    A: App<R>,
    R: IndigoRenderer,
    Self: Sized,
{
    fn handle_event(
        &mut self,
        _ctx: &mut IndigoContext<'_, '_, A, Self, R>,
        _event: ViewEvent,
    ) -> Result<(), IndigoError<R::ErrorMessage>>;
}

pub trait ViewWrapperTrait<A: App<R>, R: IndigoRenderer> {
    /// Updates the underlying View<A>
    fn update(&mut self, app: &mut IndigoApp<'_, A, R>) -> Result<(), IndigoError<R::ErrorMessage>>;
    fn render_view(
        &mut self,
        _window_size: (u32, u32),
        _app: &mut IndigoApp<'_, A, R>
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>>;
}

pub struct ViewWrapper<A, V, R> {
    view: V,
    ui_tree: UiTree<A, V, R>,
}

impl<'a, A, V, R> ViewWrapper<A, V, R>
where
    A: App<R>,
    V: View<A, R>,
    R: IndigoRenderer,
{
    pub fn new(mut view: V, app: &mut IndigoApp<'_, A, R>) -> ViewWrapper<A, V, R> {
        let mut ui_tree = UiTree::<A, V, R>::default();

        let mut context = IndigoContext { 
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
    R: IndigoRenderer + 'static,
{
    /// Inits all uninitialized widgets, updates them and then updates the underlying view
    fn update(&mut self, app: &mut IndigoApp<'_, A, R>) -> Result<(), IndigoError<R::ErrorMessage>> {
        // Not sure if this actually copies the pending_init vec but it definitely doesnt have to
        // maybe theres a better solution than .clone()?
        let mut pending_init = self.ui_tree.pending_init.clone();

        let mut context = IndigoContext { 
            app, 
            ui_tree: &mut self.ui_tree,
            current: NodeType::Root,
        };

        self.view.handle_event(&mut context, ViewEvent::Update)?;

        let handles = self.ui_tree.get_all_handles().collect::<Vec<_>>();

        for handle in &handles {
            //move the widget out to avoid aliasing refs
            let _: Result<(), IndigoError<R::ErrorMessage>> = self.ui_tree.run_on_moved_out(
                &handle, 
                |mut ui_tree, widget| {
                    let mut context = IndigoContext { 
                        app, 
                        ui_tree: &mut ui_tree,
                        current: NodeType::Handle(handle.clone())
                    };

                    if pending_init.contains(&handle) {
                        if let Err(err) = widget.handle_event(&mut context, &mut self.view, WidgetEvent::Init) {
                            return Err(err);
                        }

                        pending_init.drain_filter(|h| *h == *handle);
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
        app: &mut IndigoApp<'_, A, R>,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {

        let mut command_vec = Vec::new();

        //Get handles of root children
        let handles = self.ui_tree.get_all_handles()
            .filter(|handle| match self.ui_tree.parent_arena.vec[handle.index].unwrap() {
                NodeType::Handle(_) => false,
                NodeType::Root => true
            })
            .collect::<Vec<_>>();

        for handle in handles {
            if app.render_cache.get(&handle).is_none() {
                self.ui_tree.run_on_moved_out(&handle, |mut ui_tree, widget| {
                    let mut context = IndigoContext {
                        app,
                        ui_tree: &mut ui_tree,
                        current: NodeType::Handle(handle)
                    };
    
                    widget.handle_event(
                        &mut context,
                        &mut self.view, 
                        WidgetEvent::Render { layout: Layout {
                            origin: (0.0, 0.0, 0.0),
                            available_space: (window_size.0 as f32, window_size.1 as f32)
                        }}
                    )
                })?;
            }

            command_vec.append(&mut app.render_cache
                .get(&handle)
                .expect("Render event didnt submit a command vec")
                .clone());
        }


        Ok(command_vec)
    }
}
