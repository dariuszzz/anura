use std::rc::Rc;

use graphics::WgpuRenderer;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::{view::{View, ViewWrapper, ViewWrapperTrait}, input::InputManager, event::{IndigoResponse, AppEvent}, graphics::{Renderer, self}, error::IndigoError};

pub trait App<R> {
    fn handle_event(&mut self, _event: AppEvent) -> IndigoResponse {
        IndigoResponse::Noop
    }
}

pub struct IndigoApp<A, R> {
    app: A,
    views: Vec<Box<dyn ViewWrapperTrait<A, R>>>,

    running: bool,

    renderer: R,

    input_manager: InputManager,
    window: Rc<Window>,
}

#[cfg(feature = "wgpu-renderer")]
impl<A> IndigoApp<A, WgpuRenderer> 
    where 
        A: App<WgpuRenderer> + 'static
{
    pub async fn with_default_renderer(app: A, window: Rc<Window>) -> Self {
        
        let renderer = WgpuRenderer::new(&window).await;
        
        Self::with_renderer(app, window, renderer).await
    }

} 

impl<A, R> IndigoApp<A, R>
where 
    A: App<R> + 'static, 
    R: Renderer + 'static
{

    pub async fn with_renderer(app: A, window: Rc<Window>, renderer: R) -> Self {
        let mut this = Self {
            app,
            views: Vec::new(),
            running: true,
            renderer,
            input_manager: InputManager::default(),
            window,
        };

        this.init();
        this
    }

    fn init(&mut self) {
        self.app.handle_event(AppEvent::Init);

        self.renderer.setup_camera(
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            -10.0,
            10.0
        );
    }

    /// Pushes a new view onto the view stack
    pub fn push_view<V>(&mut self, view: V)
    where
        V: View<A, R> + 'static
    {
        let wrapped_view = ViewWrapper::new(view, &mut self.app);
        let boxed = Box::new(wrapped_view);

        self.views.push(boxed);
    }

    /// Pops the current view off the view stack
    pub fn pop_view(&mut self) {
        self.views.pop();
    }

    /// Updates the current view
    pub fn update(&mut self) {
        let len = self.views.len();
        if len == 0 {
            return;
        };

        let view = self.views.get_mut(len - 1).unwrap();
        view.update(&mut self.app);
    }

    pub fn render(&mut self) -> Result<(), IndigoError<R::ErrorMessage>> {
        self.renderer.render()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.on_window_resize(new_size);
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
                Err(IndigoError::FatalError { msg }) => {
                    //TODO: handle fatal errors differently (maybe just panic?)
                    *control_flow = ControlFlow::Exit; 
                    eprintln!("{msg:?}")
                },
                _ => {}
                // Err(wgpu::SurfaceError::Lost) => self.resize(self.window.inner_size()),
            },
            Event::MainEventsCleared => {
                if self.running {
                    self.update();
                }
                
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
            WindowEvent::ReceivedCharacter(character) => self.input_manager.last_received_char = Some(*character),
            WindowEvent::CloseRequested => self.exit(control_flow),
            WindowEvent::Resized(physical_size) => self.resize(*physical_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.resize(**new_inner_size),
            WindowEvent::CursorMoved { position, .. } => self.input_manager.update_mouse_pos(position),
            WindowEvent::MouseInput { state, button, .. } => self.input_manager.update_mouse_button(state, button),
            WindowEvent::MouseWheel { delta, phase, .. } => {}
            _ => {}
        }
    }

    fn exit(&mut self, control_flow: &mut ControlFlow) {

        self.app.handle_event(AppEvent::Exit);

        self.running = false;

        *control_flow = ControlFlow::Exit;
    }
}
