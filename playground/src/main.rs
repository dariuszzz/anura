use std::{rc::Rc, fs, marker::PhantomData, path::Path};

mod app;
mod testwidget;
mod view;

use app::*;
use testwidget::*;
use view::*;

use indigo::prelude::*;

use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

// use indigo::{
//     app::{App, IndigoApp},
//     uitree::{UiTree, WidgetHandle},
//     view::View,
//     widget::{TextWidget}, IndigoEvent, IndigoResponse,
// };

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_min_inner_size(PhysicalSize::new(800, 600))
        .with_title("Playground")
        .build(&event_loop)
        .unwrap();

    let window = Rc::new(window);

    pollster::block_on(run(window, event_loop));
}


async fn run(window: Rc<Window>, event_loop: EventLoop<()>) {
    // let instance = wgpu::Instance::new(wgpu::Backends::all());
    // let surface = unsafe { instance.create_surface(window.as_ref()) };

    let app = PlaygroundApp::new("siema");

    let mut indigo_app = IndigoApp::with_default_renderer(app, window).await;

    indigo_app.push_view(MainView::default());

    event_loop.run(move |event, _, control_flow| {
        indigo_app.handle_events(event, control_flow);
    });
}

// trait HasTextHandle {
//     fn get_handle(&self) -> &WidgetHandle<TextWidget>;
// }

// impl HasTextHandle for MainView {
//     fn get_handle(&self) -> &WidgetHandle<TextWidget> {
//         &self.text_handle
//     }
// }

// struct CustomWidget<T> {
//     field: T
// }

// impl<A, V> Widget<A, V> for CustomWidget<i32>
// where
//     A: App + 'static,
//     V: View<A> + HasTextHandle + 'static
// {
//     fn update(&mut self, _app: &mut A, _view: &mut V, _ui_tree: &mut UiTree<A, V>) {
//         self.field += 1;

//         let text_widget = _ui_tree.get_mut(_view.get_handle()).unwrap();
//         text_widget.text = format!("Licznik {}", self.field);
//     }
//     fn init(&mut self, _app: &mut A, _view: &mut V, _ui_tree: &mut UiTree<A, V>) {}
// }
