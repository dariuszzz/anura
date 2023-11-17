// I think these should be non exhaustive since they will be matched
// "client" side and handling them is optional

use crate::{widget::Layout};


// I think this distinction between view, app and widget events is pointless
// especially between view and widget, i believe these could be merged since
// views and widgets will handle the same events afaik 
// app however wont have that many events other than Init Update and Exit (maybe some kind of 
// input event if i decide to add one or minimize/maximize window but these two could be view events
// as well in order to allow for view specific logic)
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
}