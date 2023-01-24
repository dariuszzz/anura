use crate::{
    app::App,
    context::IndigoContext,
    event::{IndigoResponse, ViewEvent, WidgetEvent},
    graphics::IndigoRenderer,
    prelude::{IndigoError, Layout},
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
        _ctx: &mut IndigoContext<'_, A, Self, (), R>,
        _event: ViewEvent,
    ) -> IndigoResponse {
        IndigoResponse::Noop
    }
}

pub trait ViewWrapperTrait<A: App<R>, R: IndigoRenderer> {
    /// Updates the underlying View<A>
    fn update(&mut self, app: &mut A);
    fn render_view(
        &self,
        _renderer: &mut R,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>>;
}

pub struct ViewWrapper<A, V, R> {
    view: V,
    ui_tree: UiTree<A, V, R>,
}

impl<'a, A, V, R> ViewWrapper<A, V, R>
where
    A: App<R>,
    V: View<A, R> + 'a,
    R: IndigoRenderer,
{
    pub fn new(mut view: V, app: &'a mut A) -> ViewWrapper<A, V, R> {
        let mut ui_tree = UiTree::<A, V, R>::default();

        let ctx = &mut IndigoContext {
            app,
            view: &mut (),
            ui_tree: &mut ui_tree,
        };

        view.handle_event(ctx, ViewEvent::Init);

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
    fn update(&mut self, app: &mut A) {
        // Not sure if this actually copies the pending_init vec but it definitely doesnt have to
        // maybe theres a better solution than .clone()?
        let mut pending_init = self.ui_tree.pending_init.clone();

        // let mut handles: Vec<&UntypedHandle> = self.ui_tree.get_handles();

        let ctx = &mut IndigoContext {
            app,
            view: &mut (),
            ui_tree: &mut self.ui_tree,
        };

        self.view.handle_event(ctx, ViewEvent::Update);

        let handles = self.ui_tree.get_all_handles();

        handles.iter().for_each(|handle| {
            //move the widget out to avoid aliasing refs
            self.ui_tree.run_on_moved_out(handle, |ui_tree, widget| {
                let ctx = &mut IndigoContext::<A, V, V, R> {
                    app,
                    view: &mut self.view,
                    ui_tree,
                };

                if pending_init.contains(handle) {
                    widget.handle_event(ctx, WidgetEvent::Init { index: 0 }); //TODO: just put the index in ctx

                    pending_init.drain_filter(|h| *h == *handle);
                }

                widget.handle_event(ctx, WidgetEvent::Update);
            });
        });

        self.ui_tree.pending_init = pending_init;
    }

    fn render_view(
        &self,
        renderer: &mut R,
    ) -> Result<Vec<R::RenderCommand>, IndigoError<R::ErrorMessage>> {
        //Idk if calculating the capacity and only then making the command vec is a good idea
        //but i feel like this would be more performant when there are many commands
        let mut command_vecs = Vec::with_capacity(self.ui_tree.children_arena.vec.len());
        let mut total_commands = 0;

        //TODO: rework
        let handles = self.ui_tree.get_all_handles();

        for handle in handles {
            let widget = self.ui_tree.get_untyped_ref(handle).unwrap();

            let widget_commands = widget.generate_mesh(Layout {}, renderer)?;
            total_commands += widget_commands.len();

            command_vecs.push(widget_commands);
        }

        let mut commands = Vec::with_capacity(total_commands);

        for command_vec in &mut command_vecs {
            commands.append(command_vec);
        }

        Ok(commands)
    }
}
