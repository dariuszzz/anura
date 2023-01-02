#![feature(drain_filter)]
#![feature(hash_drain_filter)]
#![feature(trait_upcasting)]

pub mod app;
pub mod view;
pub mod widget;
pub mod arena;
pub mod uitree;
pub mod input;
pub mod error;
pub mod drawable;
pub mod event;
pub mod handle;
pub mod context;

pub mod prelude {
    pub use super::handle;
    pub use super::event;
    pub use super::error;
    pub use super::input;
    pub use super::uitree;
    pub use super::widget;
    pub use super::app;
    pub use super::view;
    pub use super::context;
}