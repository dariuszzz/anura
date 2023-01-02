pub mod text;
use std::collections::HashMap;

pub use text::*;

pub mod quad;
pub use quad::*;
use wgpu::{Queue, RenderPipeline, Instance, Surface, TextureFormat, Device};
use winit::window::Window;


#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_arr(arr: [f32; 4]) -> Self {
        Color {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }

    pub fn clamp(&self) -> Color {
        Color {
            r: if self.r > 1.0 { self.r / 255.0 } else { self.r },
            g: if self.g > 1.0 { self.g / 255.0 } else { self.g },
            b: if self.b > 1.0 { self.b / 255.0 } else { self.b },
            a: if self.a > 1.0 { self.a / 255.0 } else { self.a },
        }
    }

    pub fn to_arr(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}




pub trait Drawable {
    fn draw(&self, _renderer: usize) {

    }
}

pub struct Renderer {
    pub surface: Surface,
    pub swapchain_format: TextureFormat,
    pub queue: Queue,
    pub device: Device,
    pub pipeline_cache: HashMap<String, RenderPipeline>, 
}

impl Renderer {
    pub async fn new(window: &Window, instance: Instance) -> Self {

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: true
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Indigo device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None
        ).await.expect("Couldn't create indigo device");

        let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

        let pipeline_cache = HashMap::new();

        let this = Self {
            queue,
            device,
            surface,
            swapchain_format,
            pipeline_cache
        };

        // this.create_default_pipeline();

        this
    }
}

