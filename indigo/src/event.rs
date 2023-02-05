// I think these should be non exhaustive since they will be matched
// "client" side and handling them is optional

use crate::{widget::Layout};

#[non_exhaustive]
pub enum AppEvent {
    Init,
    Update,
    Exit,
}

#[non_exhaustive]
pub enum ViewEvent {
    Init,
    Update,
}

#[non_exhaustive]
pub enum WidgetEvent {
    Init,
    Update,
    Render { layout: Layout }
}