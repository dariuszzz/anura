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


macro_rules! debug {
    ($format:expr, $($expression:expr),+) => {
        println!("[{}:{}] {}", file!(), line!(), format!($format, $($expression),+))
    };
    ($msg:expr) => {
        println!("[{}:{}] {}", file!(), line!(), $msg)
    };
}

pub(crate) use debug;