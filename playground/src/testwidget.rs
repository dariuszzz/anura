

use super::*;

#[derive(Default)]
pub struct TestingWidget {
    pub text: String,
}

impl<A, V> Widget<A, V, WgpuRenderer> for TestingWidget
where
    A: App<WgpuRenderer>,
    V: View<A, WgpuRenderer>,
{

    fn handle_event(
        &mut self,
        _ctx: &mut AnuraContext<'_, '_, A, V, WgpuRenderer>,
        _view: &mut V,
        _event: WidgetEvent,
    ) -> Result<(), AnuraError<<WgpuRenderer as AnuraRenderer>::ErrorMessage>> {
        match _event {
            _WidgetEvent => {
                                /*
                if _ctx.clicked == _ctx.self {
                    _ctx.push_view(MainView {
                        siema
                    })
                }

                _ctx.try_pop_view();

                if _ctx.focused == _ctx.self {

                    widget_hierarchy! {
                        TableWidget {
                            rows: 5,
                            cols: 5,
                        } [
                            #siema_text TableWidget { text: "siema" }
                            #article ArticleWidget {
                                heading: TextWidget { text: "Elo" },
                                description: TextWidget { text: "Witam" }
                            },
                            #inner_table TableWidget { rows: 2, cols: 2 }
                            [
                                #inner_table_image ImageWidget { path: "img.png" }
                            ]
                            .cell(1, 0, #inner_table_image) 
                        ]
                        .cell(2, 0, #siema_text)
                        .cell(2, 1, #article)
                        .cell(3, 4, #inner_table) 
                    }

                    turns into 

                    let table_handle = ctx.ui_tree.reserve_handle();
                    let cell2_0_handle = ctx.ui_tree.reserve_handle();
                    let cell2_1_handle = ctx.ui_tree.reserve_handle();
                    let heading_handle = ctx.ui_tree.reserve_handle();
                    let description_handle = ctx.ui_tree.reserve_handle();
                    let cell3_4_handle = ctx.ui_tree.reserve_handle();
                    let cell3_4_1_0_handle = ctx.ui_tree.reserve_handle();

                    let table = TableWidget { rows: 5, cols: 5 };
                    
                    let cell2_0 = TextWidget { "siema" };
                    let cell2_1 = ArticleWidget {
                        heading: heading_handle,
                        description: desc_handle
                    };

                    let heading = TextWidget { "elo" };
                    let description = TextWidget { "Witam" };

                    let cell3_4 = TableWidget { rows: 2, cols: 2 };
                    let cell3_4_1_0 = ImageWidget { path: "img.png" };

                    cell3_4.cell(1, 0, cell3_4_1_0_handle)

                    table.cell(2, 0, cell2_0_handle);
                    table.cell(2, 1, cell2_1_handle);

                    ctx.ui_tree.insert_with_handle(table_handle, table, ctx.self);
                    ctx.ui_tree.insert_with_handle(cell2_0_handle, cell2_0, table_handle);
                    ctx.ui_tree.insert_with_handle(cell2_1_handle, cell2_1, table_handle);
                    ctx.ui_tree.insert_with_handle(heading_handle, heading, cell2_1);
                    ctx.ui_tree.insert_with_handle(description_handle, desc_handle, cell2_1);
                    ctx.ui_tree.insert_with_handle(cell3_4_handle, cell3_4, table_handle);
                    ctx.ui_tree.insert_with_handle(cell3_4_1_0_handle, cell3_4_1_0, cell3_4_handle);



                }
                */
            }
        }

        Ok(())
    }
}

impl TestingWidget {
    fn generate_mesh<A, V>(
        &self,
        _ctx: &mut AnuraContext<'_, '_, A, V, WgpuRenderer>,
        _layout: Anura::widget::Layout,
    ) -> Result<
        Vec<<WgpuRenderer as AnuraRenderer>::RenderCommand>,
        AnuraError<<WgpuRenderer as AnuraRenderer>::ErrorMessage>,
    > 
    where 
        A: App<WgpuRenderer>,
        V: View<A, WgpuRenderer>
    {
        Ok(vec![/*command*/])
    }
}
