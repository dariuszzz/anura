#[macro_use]
pub mod camera;
pub mod mesh;
pub mod renderer;
pub mod shader;
pub mod uniform;
pub mod vertex;
pub mod texture;

//Reexport wgpu
pub use wgpu;

#[allow(unused_macros)]
#[cfg(debug_assertions)]
macro_rules! debug {
    ($format:expr, $($expression:expr),+) => {
        println!("[{}:{}] {}", file!(), line!(), format!($format, $($expression),+))
    };
    ($msg:expr) => {
        println!("[{}:{}] {}", file!(), line!(), $msg)
    };
}

#[allow(unused_macros)]
#[cfg(not(debug_assertions))]
macro_rules! debug {
    () => {};
}

#[allow(unused_imports)]
pub(crate) use debug;