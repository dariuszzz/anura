use std::rc::Rc;

use wgpu::Instance;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::{view::{View, ViewWrapper, ViewWrapperTrait}, input::InputManager, drawable::Renderer, event::{IndigoResponse, AppEvent}};

pub trait App {
    fn handle_event(&mut self, _event: AppEvent) -> IndigoResponse {
        IndigoResponse::Noop
    }
}

pub struct IndigoApp<A: App> {
    app: A,
    views: Vec<Box<dyn ViewWrapperTrait<A>>>,

    running: bool,

    renderer: Renderer,
    input_manager: InputManager,
    window: Rc<Window>,
}

impl<A: App + 'static> IndigoApp<A> {
    pub async fn new(mut app: A, window: Rc<Window>, instance: Option<Instance>) -> IndigoApp<A> {

        let instance = instance.unwrap_or(
            wgpu::Instance::new(wgpu::Backends::all())
        );

        app.handle_event(AppEvent::Init);

        Self {
            app,
            views: Vec::new(),
            running: true,
            renderer: Renderer::new(&window, instance).await,
            input_manager: InputManager::default(),
            window,
        }
    }

    /// Pushes a new view onto the view stack
    pub fn push_view<V>(&mut self, view: V)
    where
        V: View<A> + 'static
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

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // self.config.width = new_size.width;
            // self.config.height = new_size.height;
            // self.surface.configure(&self.device, &self.config);
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
                Err(wgpu::SurfaceError::Lost) => self.resize(self.window.inner_size()),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{e:?}"),
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
