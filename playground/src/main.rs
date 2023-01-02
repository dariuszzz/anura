use std::{rc::Rc, fs, marker::PhantomData};

use indigo::{app::{App, IndigoApp}, uitree::{UiTree}, widget::{TextWidget, VerticalContainer}, view::View, event::{IndigoResponse, ViewEvent, AppEvent, WidgetEvent}, handle::{UntypedHandle, TypedHandle, ParentNode}, drawable::{Color, Text}, context::IndigoContext};
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

#[derive(Default)]
struct PlaygroundApp {
    _text: String,
}

impl PlaygroundApp {
    pub fn new(text: &str) -> Self {
        Self {
            _text: text.to_owned(),
        }
    }
}

impl App for PlaygroundApp {
    fn handle_event(&mut self, event: AppEvent) -> IndigoResponse {
        match event {
            AppEvent::Init => println!("App Init"),
            AppEvent::Exit => println!("App Close"),
            _ => {}
        };

        IndigoResponse::Noop
    }
}

#[derive(Default)]
struct MainView<A> {
    _marker: PhantomData<A>,
    // pub num: i32,
    pub handles: Vec<TypedHandle<TextWidget>>,
}

#[allow(unused_variables)]
impl<A: App> MainView<A> {
    fn init(&mut self, ui_tree: &mut UiTree<A, Self>) {

        let container = ui_tree.insert(
            VerticalContainer {
                color: Color::from_arr([0.2, 0.5, 0.1, 0.8])
            },
            ParentNode::Root
        );  

        let file = fs::read_to_string("./input.txt").expect("No input file");
        
        self.handles = file.lines()
            .map(|line| 
                ui_tree.insert(
                    TextWidget {
                        text: line.into(),
                        index: None,
                    },
                    &container
                )
            )
            .collect();


        println!("{:?}", self.handles);
        /*
        
        self.handles.iter().into_
        
        */

        // Implement this macro 
        //file.lines()
        //  .for_each(|line| rsi! { ui_tree,
        //      <TextWidget
        //          text: line.into()
        //      >
        //  })

    }

    fn update(&mut self, ctx: &mut IndigoContext<A, Self, ()>){


        // #[feature(try_blocks)]
         
        // let str: &str = try { ctx.get_widget::<TextWidget>(idx)?.title } ;
        // ctx.get_children(); // Vec<Idx>
        // ctx.get_parent::<VerticalContainer>(idx)?;
        // ctx.get_parent(idx); // Box<dyn Widget<...>> 

        self.handles.iter()
            .filter_map(|h| 
                ctx.ui_tree.get_typed_ref(h)
            )
            .for_each(|widget| 
                println!("{:?}", widget.text)
            );

        // self.handles.iter()
        //     .map(|handle| handle.into().get(&ctx.ui_tree))
        //     // Filter off every widget that isnt a TextWidget
        //     .filter_map(|wrapper| 
        //         Some((
        //             wrapper.get_widget_ref().downcast_ref::<TextWidget>()?,
        //             wrapper.get_children_ref(),
        //             wrapper.get_parent_ref()
        //         ))
        //     )
        //     // .map(|(text_widget, _, _)| text_widget.text.clone())
        //     // .collect::<Vec<_>>();
        //     .for_each(|(text_widget, children, parent)| {
        //         println!("Text: {}, Children: {children:?}, Parent: {parent:?}", text_widget.text);
        //         // if let Some(parent) = parent {
        //         //     if let Some(parent) = ctx.ui_tree.widget_arena.get(*parent) {
        //         //         // parent.handle_event(ctx, WidgetEvent::Update);
        //         //     }
        //         // }
        //     }); 


        // for handle in self.handles.iter() {
        //     let widget = ctx.ui_tree.get(handle).unwrap();
        //     // WidgetWrapper is Deref<Target=Widget> if Typed and Deref<Target=dyn Widget<A,V>> if Untyped
        //     let children = widget.children;
        //     let parent = widget.parent;

        //     println!("Text: {}, Children: {children:?}, Parent: {parent:?}", widget.text);

        // }


        // self.handles.iter()
        //     .with_context
        //     .for_each()
        // println!("{texts:?}");
        // ctx.ui_tree.insert(TextWidget { text: "".to_string(), index: None }, None::<&TypedHandle<()>>);
    }
}

#[allow(unused_variables)]
impl<A: App> View<A> for MainView<A> {

    fn handle_event(
        &mut self,
        ctx: &mut IndigoContext<A, Self, ()>,
        event: ViewEvent
    ) -> IndigoResponse {

        match event {
            ViewEvent::Init => self.init(ctx.ui_tree),
            ViewEvent::Update => self.update(ctx),
            _ => {}
        }

        IndigoResponse::Noop
    }
}

// pub struct TestingWidget {
//     pub text: String,
//     pub text_handle: WidgetHandle<TextWidget>,
// }

// impl<A: App, V: View<A>> Widget<A, V> for TestingWidget {
//     fn init(&mut self, _app: &mut A, _view: &mut V, ui_tree: &mut UiTree<A, V>, _id: usize) {
//         self.text_handle = ui_tree.insert(
//             TextWidget {
//                 text: String::new(),
//                 fun: Box::new(|| { self.text.clone() })
//             },
//             None,
//         );
//     }
// }

async fn run(window: Rc<Window>, event_loop: EventLoop<()>) {
    // let instance = wgpu::Instance::new(wgpu::Backends::all());
    // let surface = unsafe { instance.create_surface(window.as_ref()) };

    let app = PlaygroundApp::new("siema");
    let mut indigo_app = IndigoApp::new(app, window, None).await;

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
