use super::*;

pub struct TestingWidget {
    pub text: String,
}

impl<A, V> Widget<A, V, WgpuRenderer> for TestingWidget
where
    A: App<WgpuRenderer>,
    V: View<A, WgpuRenderer>,
{
    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            text: String::new(),
        }
    }

    fn handle_event(
        &mut self,
        _ctx: &mut IndigoContext<'_, A, V, V, WgpuRenderer>,
        _event: WidgetEvent,
    ) -> IndigoResponse {
        IndigoResponse::Noop
    }

    fn generate_mesh(
        &self,
        _layout: indigo::widget::Layout,
        _renderer: &mut WgpuRenderer,
    ) -> Result<
        Vec<<WgpuRenderer as IndigoRenderer>::RenderCommand>,
        IndigoError<<WgpuRenderer as IndigoRenderer>::ErrorMessage>,
    > {
        Ok(vec![/*command*/])
    }
}
