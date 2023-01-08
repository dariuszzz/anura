use std::collections::HashMap;

use wgpu::{Queue, RenderPipeline, Instance, Surface, TextureFormat, Device, BindGroup, TextureView, SurfaceConfiguration};
use wgpu::util::DeviceExt;

use crate::{vertex::Vertex, camera::{Camera, CameraUniform}};



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






pub struct RenderingContext {
    pub surface: Surface,
    pub swapchain_format: TextureFormat,
    pub queue: Queue,
    pub device: Device,
    pub pipeline: RenderPipeline,
    pub config: SurfaceConfiguration,
}

impl RenderingContext {
    pub async fn new<W>(
        window_size: impl Into<[u32; 2]>, 
        window: &W, 
    ) -> Self 
        where W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle
    {
        let window_size = window_size.into();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

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

        let swapchain_format = surface.get_supported_formats(&adapter)
            .into_iter()
            .filter(|format| {
                let desc = format.describe();

                desc.srgb
        }).next().expect("Couldn't find appropriate surface");

        
        let config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size[0],
            height: window_size[1],
            present_mode: wgpu::PresentMode::Mailbox
        };
    
        surface.configure(&device, &config);


        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("indigo renderer shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/main.wgsl").into())
            }
        );

        let camera_bgl = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera bg"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
            }
        );

        let rp_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("rp layout"),
                bind_group_layouts: &[
                    &camera_bgl
                ],
                push_constant_ranges: &[],
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("render pipeline"),
                layout: Some(&rp_layout), 
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::desc()
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: swapchain_format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    operation: wgpu::BlendOperation::Add,
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,//Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }
        );

        Self {
            queue,
            device,
            surface,
            swapchain_format,
            pipeline,
            config,
        }
    }

    pub fn temp_render(&self, camera_bg: &wgpu::BindGroup) -> Result<(), wgpu::SurfaceError> {
        
        let output = self.surface.get_current_texture()?;
        let output_tex = output.texture.create_view(&wgpu::TextureViewDescriptor::default());


        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("basic renderer")
            }
        );

        let vertices = [
            Vertex { pos: [0.0, 0.0, 0.0], tint_color: [0.0, 0.0, 0.0, 1.0] },
            Vertex { pos: [self.config.width as f32, 0.0, 0.0], tint_color: [1.0, 0.0, 0.0, 1.0] },
            Vertex { pos: [self.config.width as f32, self.config.height as f32, 0.0], tint_color: [0.0, 1.0, 0.0, 1.0] },
            Vertex { pos: [0.0, self.config.height as f32, 0.0], tint_color: [0.0, 0.0, 1.0, 1.0] },
        ];

        let indices = (0..vertices.len()).step_by(4).enumerate().map(|(_, i)| {
            let i = i as u16;
            vec![i, i+1, i+2, i+2, i+3, i]
        }).flatten().collect::<Vec<_>>();

        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX
            }
        );

        let index_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("render pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view: &output_tex,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            }
                        })
                    ],
                    depth_stencil_attachment: None,
                }
            );

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &camera_bg, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        
        output.present();

        Ok(())
    }

    pub fn new_camera(
        &mut self,
        pos: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        znear: f32,
        zfar: f32
    ) -> Camera { 
        let uniform = CameraUniform::new();

        let buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bg_layout = self.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("OrthoCamera bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
            }
        );  

        let bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bg_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding()
                    }
                ],
                label: Some("Ortho bind group")
            }
        );

        let mut camera = Camera {
            pos,
            target,
            up,
            zfar,
            znear,
            uniform,
            buffer,
            bg_layout,
            bind_group
        };

        camera.update(&self.queue, &self.config);

        camera
    }

    pub fn update_surface(&mut self, new_size: (u32, u32)) {
        self.config.width = new_size.0;
        self.config.height = new_size.1;
        self.surface.configure(&self.device, &self.config);
    }
}

