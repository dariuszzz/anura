use std::error::Error;

use crate::{
    app::{App, IndigoApp},
    context::IndigoContext,
    event::{ViewEvent, WidgetEvent},
    graphics::IndigoRenderer,
    prelude::{IndigoError, Layout, ParentNode},
    uitree::UiTree,
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
    fn update(&mut self, app: &mut IndigoApp<'_, A, R>);
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
    fn update(&mut self, app: &mut IndigoApp<'_, A, R>) {
        // Not sure if this actually copies the pending_init vec but it definitely doesnt have to
        // maybe theres a better solution than .clone()?
        let mut pending_init = self.ui_tree.pending_init.clone();

        let mut context = IndigoContext { 
            app, 
            ui_tree: &mut self.ui_tree 
        };

        self.view.handle_event(&mut context, ViewEvent::Update);

        let handles = self.ui_tree.get_all_handles().collect::<Vec<_>>();

        handles.iter().for_each(|handle| {
            //move the widget out to avoid aliasing refs
            self.ui_tree.run_on_moved_out(&handle, |mut ui_tree, widget| {
                let mut context = IndigoContext { 
                    app, 
                    ui_tree: &mut ui_tree 
                };

                if pending_init.contains(&handle) {
                    widget.handle_event(&mut context, &mut self.view, WidgetEvent::Init);

                    pending_init.drain_filter(|h| *h == *handle);
                }

                widget.handle_event(&mut context, &mut self.view, WidgetEvent::Update);
            });
        });

        self.ui_tree.pending_init = pending_init;
    }

    fn render_view(
        &mut self,
        window_size: (u32, u32),
        app: &mut IndigoApp<'_, A, R>,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        //Idk if calculating the capacity and only then making the command vec is a good idea
        //but i feel like this would be more performant when there are many commands
        let mut command_vecs = Vec::with_capacity(self.ui_tree.children_arena.vec.len());
        let mut total_commands = 0;

        //Get handles of root children
        let handles = self.ui_tree.get_all_handles()
            .filter(|handle| match self.ui_tree.parent_arena.vec[handle.index].unwrap() {
                ParentNode::Handle(_) => false,
                ParentNode::Root => true
            })
            .collect::<Vec<_>>();



        for handle in handles {
            let widget_commands = self.ui_tree.run_on_moved_out(&handle, |mut ui_tree, widget| {
                let mut context = IndigoContext {
                    app,
                    ui_tree: &mut ui_tree,
                };

                widget.handle_event(
                    &mut context,
                    &mut self.view, 
                    WidgetEvent::Render { layout: Layout {
                        origin: (0.0, 0.0, 0.0),
                        available_space: (window_size.0 as f32, window_size.1 as f32)
                    }}
                );
            });

            let widget = self.ui_tree.get_untyped_ref(&handle).unwrap();
            // total_commands += widget_commands.len();

            // command_vecs.push(widget_commands);
        }

        let mut commands = Vec::with_capacity(total_commands);

        for command_vec in &mut command_vecs {
            commands.append(command_vec);
        }

        Ok(commands)
    }
}
