use std::rc::Rc;

use graphics::WgpuRenderer;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::{
    error::IndigoError,
    event::{AppEvent, IndigoResponse},
    graphics::{self, IndigoRenderer},
    input::InputManager,
    view::{View, ViewWrapper, ViewWrapperTrait}, font::FontManager,
};

pub trait App<R> {
    fn handle_event(&mut self, _event: AppEvent) -> IndigoResponse {
        IndigoResponse::Noop
    }
}

pub struct IndigoApp<'a, A, R: IndigoRenderer> {
    app: A,
    views: Vec<Box<dyn ViewWrapperTrait<A, R> + 'a>>,

    running: bool,

    renderer: R,
    font_manager: FontManager<R>,

    input_manager: InputManager,
    window: Rc<Window>,
}

#[cfg(feature = "wgpu-renderer")]
impl<'a, A> IndigoApp<'a, A, WgpuRenderer>
where
    A: App<WgpuRenderer> + 'a,
{
    pub async fn with_default_renderer(app: A, window: Rc<Window>) -> IndigoApp<'a, A, WgpuRenderer> {
        let renderer = WgpuRenderer::new(&window).await;

        Self::with_renderer(app, window, renderer).await
    }
}

impl<'a, A, R> IndigoApp<'a, A, R>
where
    A: App<R>,
    R: IndigoRenderer,
{
    pub async fn with_renderer(app: A, window: Rc<Window>, renderer: R) -> IndigoApp<'a, A, R> {

        let mut this = Self {
            app,
            views: Vec::new(),
            running: true,
            renderer,
            font_manager: FontManager::new(),
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
            10.0,
        );
    }

    /// Pushes a new view onto the view stack
    pub fn push_view<V>(&mut self, view: V)
    where
        V: View<A, R> + 'static,
        R: 'static,
        A: 'static
    {
        let wrapped_view = ViewWrapper::new(view, &mut self.app, &mut self.font_manager, &mut self.renderer);
        let boxed = Box::new(wrapped_view);

        self.views.push(boxed);
    }

    /// Pops the current view off the view stack
    pub fn pop_view(&mut self) {
        self.views.pop();
    }

    /// Updates the current view
    pub fn update(&mut self) {
        let view = self.views.last_mut();

        if let Some(curr_view) = view {
            curr_view.update(&mut self.app, &mut self.font_manager, &mut self.renderer);
        }
    }

    pub fn render(&mut self) -> Result<(), IndigoError<R::ErrorMessage>> {
        let view = self.views.last_mut();

        if let Some(curr_view) = view {
            let window_size = self.window.inner_size();
            let commands = curr_view.render_view(window_size.into(), &mut self.app, &mut self.renderer, &self.font_manager)?;

            self.renderer.render(commands)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let (new_width, new_height) = new_size.into();
        if new_width > 0 && new_height > 0 {
            self.renderer.on_window_resize((new_width, new_height));
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
                }
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
        self.app.handle_event(AppEvent::Exit);

        self.running = false;

        *control_flow = ControlFlow::Exit;
    }
}
