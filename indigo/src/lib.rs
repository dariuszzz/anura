#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#![feature(trait_upcasting)]
#[macro_use]

pub mod app;
pub mod arena;
pub mod context;
pub mod error;
pub mod event;
pub mod graphics;
pub mod handle;
pub mod input;
pub mod uitree;
pub mod view;
pub mod widget;
pub mod font;

pub mod prelude {
    pub use super::app::*;
    pub use super::context::*;
    pub use super::error::*;
    pub use super::event::*;
    pub use super::graphics::*;
    pub use super::handle::*;
    pub use super::input::*;
    pub use super::uitree::*;
    pub use super::view::*;
    pub use super::widget::*;
    pub use super::font::*;

    pub use winit;
}

macro_rules! debug {
    ($format:expr, $($expression:expr),+) => {
        println!("[{}:{}] {}", file!(), line!(), format!($format, $($expression),+))
    };
    ($msg:expr) => {
        println!("[{}:{}] {}", file!(), line!(), $msg)
    };
}

pub(crate) use debug;