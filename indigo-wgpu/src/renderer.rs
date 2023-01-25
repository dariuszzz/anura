use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{Hash};
use std::path::{PathBuf, Path};

use image::GenericImageView;
use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration,
    TextureFormat, 
};

use crate::mesh::{VertexLayoutInfo, Mesh};
use crate::shader::{Shader, ShaderModule};
use crate::texture::Texture;
use crate::uniform::UniformBindGroup;

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
    pub config: SurfaceConfiguration,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    //Shader memory location as key since when including a file via static theres no reason to
    //do it multiple times also avoids an expensive hash of the entire file
    pub shader_modules: HashMap<*const str, wgpu::ShaderModule>,
    pub uniform_bindings: Vec<UniformBindGroup>,
    pub textures: HashMap<PathBuf, Texture>,
    pub render_pipelines: HashMap<RenderPipelineInfo, wgpu::RenderPipeline>,
}

impl RenderingContext {
    pub async fn new<W>(window_size: impl Into<[u32; 2]>, window: &W) -> Self
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let window_size = window_size.into();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: true,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Indigo device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Couldn't create indigo device");

        let swapchain_format = surface
            .get_supported_formats(&adapter)
            .into_iter()
            .find(|format| {
                let desc = format.describe();

                desc.srgb
            })
            .expect("Couldn't find appropriate surface");

        let config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: window_size[0],
            height: window_size[1],
            present_mode: wgpu::PresentMode::AutoNoVsync,
        };

        surface.configure(&device, &config);

        //Completely arbitrary max count copied from some website lol
        //wgpu doesnt seem to have a way to query the max amount of verts per draw call
        let max_vertex_count = 65536;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            size: max_vertex_count,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("index buffer"),
            size: max_vertex_count,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            queue,
            device,
            surface,
            swapchain_format,
            // pipeline,
            config,

            vertex_buffer,
            index_buffer,
            shader_modules: HashMap::new(),
            uniform_bindings: Vec::new(),
            textures: HashMap::new(),
            render_pipelines: HashMap::new(),
        }
    }

    pub fn batch_render(
        &mut self,
        mesh: Mesh,
        shader: Shader,
        texture_paths: Vec<PathBuf>,
        uniform_datas: Vec<(Vec<u8>, wgpu::ShaderStages)>,
        output_tex: &wgpu::TextureView,
    ) {

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("basic renderer"),
            });

        let assigned_binding_ids = self.find_or_create_uniform_bindings(&uniform_datas);

        let pipeline_info = RenderPipelineInfo {
            vertex_layout: mesh.layout,
            shader,
            textures: texture_paths.clone(),
            uniform_binding_ids: assigned_binding_ids.clone(),
        };

        self.create_pipeline_if_doesnt_exist(&pipeline_info);
        let pipeline = self.render_pipelines.get(&pipeline_info).unwrap();

        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&mesh.vertices));
        self.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&mesh.indices));

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_tex,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(pipeline);

            let mut bind_group_idx = 0;
            
            for (uniform_id, binding_id ) in assigned_binding_ids.iter().enumerate() {
                let assigned_binding = self.uniform_bindings.get(*binding_id).unwrap();
                assigned_binding.update(&self.queue, &uniform_datas[uniform_id].0);
                render_pass.set_bind_group(bind_group_idx, &assigned_binding.bind_group, &[]);
                bind_group_idx += 1;
            }

            for texture_path in texture_paths.iter() {
                let texture = self.textures.get(texture_path).unwrap();
                render_pass.set_bind_group(bind_group_idx, &texture.bind_group, &[]);
                bind_group_idx += 1;
            }

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn find_or_create_uniform_bindings(
        &mut self,
        uniforms: &[(Vec<u8>, wgpu::ShaderStages)],
    ) -> Vec<usize> {
        let mut chosen_bindings = Vec::new();

        let mut uniform_sizes = uniforms
            .iter()
            .cloned()
            .map(|(data, stages)| ((std::mem::size_of::<u8>() * data.len()) as u64, stages))
            .collect::<Vec<_>>();

        //Sort from highest to lowest size in order to take up bindings with big buffers first
        //this is so low size uniforms dont take the biggest buffers which would force creation
        //of unneccessary big buffers
        uniform_sizes.sort_by(|(a, _), (b, _)| b.cmp(a));

        'outer: for (uniform_idx, (uniform_size, stages)) in uniform_sizes.into_iter().enumerate() {
            for binding_idx in 0..self.uniform_bindings.len() {
                //If the binding's buffer can accommodate the given uniform
                //and it hasnt been chosen already
                let binding = self.uniform_bindings.get(binding_idx).unwrap();

                if binding.min_size >= uniform_size && !chosen_bindings.contains(&binding_idx) {
                    chosen_bindings.push(binding_idx);
                    continue 'outer;
                }
            }

            //If there was no appropriate binding available then create a new one
            let new_binding = UniformBindGroup::new(&self.device, stages, &uniforms[uniform_idx].0);
            self.uniform_bindings.push(new_binding);
            //and add it to the chosen list
            chosen_bindings.push(self.uniform_bindings.len() - 1);
            dbg!("Created new binding");
        }

        chosen_bindings
    }

    pub fn update_surface(&mut self, new_size: (u32, u32)) {
        self.config.width = new_size.0;
        self.config.height = new_size.1;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn create_texture_if_doesnt_exist(&mut self, texture_path: &std::path::Path) {
        if self.textures.contains_key(&texture_path.to_path_buf()) { 
            return 
        }

        let image = image::open(texture_path).unwrap();

        let texture = Texture::new(&self.device, &self.queue, &image.to_rgba8(), image.dimensions());

        self.textures.insert(texture_path.to_path_buf(), texture);
        dbg!("Created new texture");

    }

    pub fn create_shader_module_if_doesnt_exist(&mut self, shader_contents: &str) {
        let shader_location = shader_contents as *const _;
        
        if self.shader_modules.contains_key(&shader_location) {
            return
        }

        let module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_contents),
                source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
            });

        self.shader_modules.insert(shader_location, module);
        dbg!("Created new shader module");

    }

    pub fn create_pipeline_if_doesnt_exist(&mut self, pipeline_info: &RenderPipelineInfo) {
        if self.render_pipelines.contains_key(pipeline_info) {
            return;
        }

        let uniform_layouts = pipeline_info
            .uniform_binding_ids
            .iter()
            .map(|idx| &self.uniform_bindings.get(*idx).unwrap().bind_group_layout);

        let texture_layouts = pipeline_info
            .textures
            .iter()
            .filter_map(|path| self.textures.get(path))
            .map(|tex| &tex.bind_group_layout);

        let layouts = uniform_layouts.chain(texture_layouts).collect::<Vec<_>>();

        let rp_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &layouts,
                push_constant_ranges: &[],
            });

        let (vert_module, frag_module) = match &pipeline_info.shader.modules {
            ShaderModule::Single { module } => {
                let module = self.shader_modules.get(module).expect("No shader found??");
                (module, module)
            }
            ShaderModule::Separate { vertex, fragment } => {
                let vert = self.shader_modules.get(vertex).expect("No shader found??");
                let frag = self
                    .shader_modules
                    .get(fragment)
                    .expect("No shader found??");
                (vert, frag)
            }
        };

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&rp_layout),
                vertex: wgpu::VertexState {
                    module: vert_module,
                    entry_point: &pipeline_info.shader.vert_entry,
                    buffers: &[pipeline_info.vertex_layout.descriptor()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: frag_module,
                    entry_point: &pipeline_info.shader.frag_entry,
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.swapchain_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
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
            });

        self.render_pipelines.insert(pipeline_info.clone(), pipeline);
        dbg!("Created new pipeline");

    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RenderPipelineInfo {
    vertex_layout: VertexLayoutInfo,
    shader: Shader,
    textures: Vec<PathBuf>,
    uniform_binding_ids: Vec<usize>,
}
