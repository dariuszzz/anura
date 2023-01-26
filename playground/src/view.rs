use std::path::PathBuf;

use super::*;

#[derive(Default)]
pub struct MainView {
    // pub num: i32,
    pub handles: Vec<TypedHandle<TextWidget>>,
}

#[allow(unused_variables)]
impl MainView {
    fn init<A: App<R>, R: IndigoRenderer>(&mut self, ui_tree: &mut UiTree<A, Self, R>) {
        let container = ui_tree.insert(VerticalContainer {}, ParentNode::Root);

        let file = fs::read_to_string("./input.txt").expect("No input file");

        self.handles = file
            .lines()
            .map(|line| {
                ui_tree.insert(
                    TextWidget {
                        text: line.into(),
                        index: None,
                        ..<TextWidget as Widget<A, Self, R>>::default()
                    },
                    &container,
                )
            })
            .collect();

        ui_tree.insert(
            Image {
                image_path: PathBuf::from("./banana.png")
            },
            &container
        );

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

    fn update<A: App<R>, R: IndigoRenderer>(&mut self, ctx: &mut IndigoContext<A, Self, (), R>) {

        // #[feature(try_blocks)]

        // let str: &str = try { ctx.get_widget::<TextWidget>(idx)?.title } ;
        // ctx.get_children(); // Vec<Idx>
        // ctx.get_parent::<VerticalContainer>(idx)?;
        // ctx.get_parent(idx); // Box<dyn Widget<...>>

        // self.handles.iter()
        //     .filter_map(|h|
        //         ctx.ui_tree.get_typed_ref(h)
        //     )
        //     .for_each(|widget|
        //         println!("{:?}", widget.text)
        //     );

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
impl<A: App<R>, R: IndigoRenderer> View<A, R> for MainView {
    fn handle_event(
        &mut self,
        ctx: &mut IndigoContext<A, Self, (), R>,
        event: ViewEvent,
    ) -> IndigoResponse {
        match event {
            ViewEvent::Init => self.init(ctx.ui_tree),
            ViewEvent::Update => self.update(ctx),
            _ => {}
        }

        IndigoResponse::Noop
    }
}
