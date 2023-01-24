use super::*;

#[derive(Default)]
pub struct PlaygroundApp {
    _text: String,
}

impl PlaygroundApp {
    pub fn new(text: &str) -> Self {
        Self {
            _text: text.to_owned(),
        }
    }
}

impl<R: IndigoRenderer> App<R> for PlaygroundApp {
    fn handle_event(&mut self, event: AppEvent) -> IndigoResponse {
        match event {
            AppEvent::Init => println!("App Init"),
            AppEvent::Exit => println!("App Close"),
            _ => {}
        };

        IndigoResponse::Noop
    }
}
