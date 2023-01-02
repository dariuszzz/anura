pub enum AppEvent {
    Init,
    Update,
    Exit
}

pub enum ViewEvent {
    Init,
    Update
}

pub enum WidgetEvent {
    Init { index: usize },
    Update,
}


pub enum IndigoResponse {
    Noop
}