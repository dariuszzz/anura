use std::rc::Rc;

use ahash::AHashMap;
use graphics::WgpuRenderer;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::{
    error::AnuraError,
    event::{AppEvent},
    graphics::{self, AnuraRenderer},
    input::InputManager,
    view::{View, ViewWrapper, ViewWrapperTrait}, font::FontManager, arena::Arena, handle::UntypedHandle,
};

pub trait App<R: AnuraRenderer>: Sized {
    fn handle_event(
        &mut self, 
        _ctx: &mut AnuraApp<'_, Self, R>, 
        _event: AppEvent
    ) -> Result<(), AnuraError<R::ErrorMessage>>;
}

#[derive(Default)]
enum CurrentView {
    #[default]
    None,
    View(usize),

    //This will be used for transitions 
    #[allow(dead_code)]
    Transition { from: usize, to: usize }
}

pub struct AnuraApp<'a, A, R: AnuraRenderer> {
    pub app: Option<A>,
    views: Arena<Box<dyn ViewWrapperTrait<A, R> + 'a>>,
    current_view: CurrentView,
    view_history: Vec<usize>,
    
    pub renderer: R,
    pub(crate) render_cache: AHashMap<UntypedHandle, Vec<R::RenderCommand>>,
    pub font_manager: FontManager<R>,
    pub input_manager: InputManager,

    window: Rc<Window>,
}

#[cfg(feature = "wgpu-renderer")]
impl<'a, A> AnuraApp<'a, A, WgpuRenderer>
where
    A: App<WgpuRenderer> + 'static,
{
    pub async fn with_default_renderer(app: A, window: Rc<Window>) -> AnuraApp<'a, A, WgpuRenderer> {
        let renderer = WgpuRenderer::new(&window).await;

        Self::with_renderer(app, window, renderer).await
    }
}

impl<'a, A, R> AnuraApp<'a, A, R>
where
    A: App<R> + 'static,
    R: AnuraRenderer + 'static,
{
    pub async fn with_renderer(app: A, window: Rc<Window>, renderer: R) -> AnuraApp<'a, A, R> {

        let mut this = Self {
            app: Some(app),
            views: Arena::new(),
            current_view: CurrentView::None,
            view_history: Vec::new(),
            renderer,
            render_cache: AHashMap::new(),
            font_manager: FontManager::new(),
            input_manager: InputManager::default(),
            window,
        };

        this.init();
        this
    }

    fn init(&mut self) {
        let mut app = self.app.take().unwrap();

        app.handle_event(self, AppEvent::Init);

        self.app = Some(app);

        self.renderer.setup_camera(
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            -10.0,
            10.0,
        );
    }

    /// Pushes a new view onto the view stack
    pub fn push_view<V>(&mut self, view: V)
    where
        V: View<A, R> + 'static,
    {
        
        let wrapped_view = ViewWrapper::new(view, self);
        let boxed = Box::new(wrapped_view);

        let id = self.views.insert(boxed);
        self.view_history.push(id);
        self.current_view = CurrentView::View(id);
    }

    pub fn get_current_view_id(&self) -> usize {
        match self.current_view {
            CurrentView::None => panic!("No view is being displayed"),
            CurrentView::View(id) => id,
            CurrentView::Transition { from: _, to } => to
        }
    }

    /// Pops the current view off the view stack
    pub fn pop_view(&mut self) -> Box<dyn ViewWrapperTrait<A, R> + 'a> {
        match self.current_view {
            CurrentView::None => panic!("No view to pop"),
            CurrentView::Transition { from: _, to: _ } => panic!("Can't pop view mid transition"),
            CurrentView::View(id) => {
                //Pop current view from history
                self.view_history.pop().unwrap();
                
                self.current_view = match self.view_history.last() {
                    None => CurrentView::None,
                    Some(last_view) => CurrentView::View(*last_view)
                };
                
                self.views.remove(id).unwrap()
            },
        }

    }

    #[allow(dead_code)]
    pub(crate) fn get_current_view(&mut self) -> &mut Box<dyn ViewWrapperTrait<A, R> + 'a> {
        let id = self.get_current_view_id();
        self.views.get_mut(id).expect("View is moved out")
    }

    pub(crate) fn run_on_moved_out_view<T, F>(&mut self, id: usize, f: F) -> T
    where 
        F: FnOnce(&mut AnuraApp<'a, A, R>, &mut Box<dyn ViewWrapperTrait<A, R> + 'a>) -> T
    {
        let mut view = self.views.vec[id].take().unwrap();

        let res = f(self, &mut view);

        self.views.vec[id] = Some(view);

        res
    }

    /// Updates the current view
    pub fn update(&mut self) {
        self.run_on_moved_out_view(self.get_current_view_id(), |mut app, view| {
            view.update(&mut app);
        });

    }

    pub fn render(&mut self) -> Result<(), AnuraError<R::ErrorMessage>> {
        let window_size = self.window.inner_size();

        let commands = self.run_on_moved_out_view(self.get_current_view_id(), |mut app, view| {
            view.render_view(window_size.into(), &mut app)
        })?;
        
        self.renderer.render(commands)?;

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let (new_width, new_height) = new_size.into();
        if new_width > 0 && new_height > 0 {
            self.renderer.on_window_resize((new_width, new_height));
            //Clear render command cache in order to avoid stretching
            self.render_cache.clear();
        }
    }

    /// Main loop of the app
    ///
    /// Handles winit events
    pub fn handle_events(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.window.id() => self.handle_window_events(event, control_flow),
            Event::RedrawRequested(_) => match self.render() {
                Ok(_) => {}
                Err(AnuraError::FatalError { msg }) => {
                    //TODO: handle fatal errors differently (maybe just panic?)
                    self.resize(self.window.inner_size());
                    // *control_flow = ControlFlow::Exit;
                    // eprintln!("{msg:?}")
                }
            },
            Event::MainEventsCleared => {
                self.update();

                self.window.request_redraw();

                self.input_manager.update_inputs();
            }
            _ => {}
        }
    }

    /// Handles just the windowevent part of winit
    pub fn handle_window_events(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => self.input_manager.update_key(keycode, state),
            WindowEvent::ReceivedCharacter(character) => {
                self.input_manager.last_received_char = Some(*character)
            }
            WindowEvent::CloseRequested => self.exit(control_flow),
            WindowEvent::Resized(physical_size) => self.resize(*physical_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.resize(**new_inner_size),
            WindowEvent::CursorMoved { position, .. } => {
                self.input_manager.update_mouse_pos(position)
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input_manager.update_mouse_button(state, button)
            }
            WindowEvent::MouseWheel { delta: _, phase: _, .. } => {},
            _ => {}
        }
    }

    fn exit(&mut self, control_flow: &mut ControlFlow) {
        let mut app = self.app.take().unwrap();

        app.handle_event(self, AppEvent::Exit);
        

        *control_flow = ControlFlow::Exit;
    }
}
