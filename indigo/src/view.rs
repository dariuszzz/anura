use crate::{
    app::App,
    event::{IndigoResponse, ViewEvent, WidgetEvent},
    uitree::UiTree, context::{IndigoContext}, handle::UntypedHandle, widget::Widget, graphics::Renderer,
};

pub trait View<A, R>
where
    A: App<R>,
    R: Renderer,
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

pub trait ViewWrapperTrait<A: App<R>, R: Renderer> {
    /// Updates the underlying View<A>
    fn update(&mut self, app: &mut A);
}

pub struct ViewWrapper<A, V, R>
where
{
    view: V,
    ui_tree: UiTree<A, V, R>,
}

impl<'a, A, V, R> ViewWrapper<A, V, R>
where
    A: App<R>,
    V: View<A, R> + 'a,
    R: Renderer
{
    pub fn new(mut view: V, app: &'a mut A) -> ViewWrapper<A, V, R> {
        let mut ui_tree = UiTree::<A, V, R>::default();

        {
            let ctx = &mut IndigoContext {
                app,
                view: &mut (),
                ui_tree: &mut ui_tree,
            };
    
            view.handle_event(ctx, ViewEvent::Init);
        }

        Self { ui_tree, view }
    }
}

impl<'a, A, V, R> ViewWrapperTrait<A, R> for ViewWrapper<A, V, R>
where
    A: App<R> + 'static,
    V: View<A, R> + 'static,
    R: Renderer + 'static
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
            ui_tree: &mut self.ui_tree
        };  

        self.view.handle_event(ctx, ViewEvent::Update);


        let handles: Vec<UntypedHandle> = self.ui_tree.widget_arena.vec.iter()
            .enumerate()
            .filter(|(idx, w)| w.is_some())
            .map(|(index, _)| UntypedHandle { index })
            .collect();

        handles.iter()
            .for_each(|handle| {
                //move the widget out to avoid aliasing refs
                let mut widget = self.ui_tree.widget_arena.vec[handle.index].take().unwrap();
                
                let ctx = &mut IndigoContext::<A, V, V, R> {
                    app,
                    view: &mut self.view,
                    ui_tree: &mut self.ui_tree
                };

                if pending_init.contains(&handle) {
                    widget.handle_event(ctx, WidgetEvent::Init { index: 0 }); //TODO: just put the index in ctx

                    pending_init.drain_filter(|h| *h == *handle);
                }

                widget.handle_event(ctx, WidgetEvent::Update);

                //move the widget back in
                self.ui_tree.widget_arena.vec[handle.index] = Some(widget);
            });
   
        self.ui_tree.pending_init = pending_init;
    }
}
