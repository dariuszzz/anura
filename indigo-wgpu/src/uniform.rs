use wgpu::util::DeviceExt;

pub struct UniformHandle {
    pub min_size: u64,
    pub stages: wgpu::ShaderStages,
}

pub struct UniformBinding {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub min_size: u64,
}

impl UniformBinding {
    pub fn new(device: &wgpu::Device, stages: wgpu::ShaderStages, uniform_contents: &[u8]) -> Self {
        let contents_size = std::mem::size_of_val(uniform_contents) as u64;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: stages,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(contents_size),
                },
                count: None,
            }],
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: uniform_contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            bind_group_layout,
            buffer,
            bind_group,
            min_size: contents_size,
        }
    }

    //Doesnt have to take &mut self but i feel like it should since its technically
    //mutating something
    pub fn update(&mut self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }
}
