use crate::texture::Texture;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    color: [f32; 4],      // 16 bytes
    use_texture: u32,     // 4 bytes
    _padding: [u32; 7],   // 28 bytes -> Total: 48 bytes
}

pub struct Material {
    pub diffuse_texture: Option<Texture>,
    pub bind_group: wgpu::BindGroup,
    pub color: [f32; 4],
    material_buffer: wgpu::Buffer,
}

impl Material {
    /// Créer un matériau avec texture
    pub fn with_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bytes: &[u8],
        label: &str,
    ) -> anyhow::Result<Self> {
        let diffuse_texture = Texture::from_bytes(device, queue, texture_bytes, label)?;

        let material_uniform = MaterialUniform {
            color: [1.0, 1.0, 1.0, 1.0],
            use_texture: 1,
            _padding: [0; 7],
        };

        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_material_buffer", label)),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = Self::create_bind_group_layout(device);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{}_bind_group", label)),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: material_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            diffuse_texture: Some(diffuse_texture),
            bind_group,
            color: [1.0, 1.0, 1.0, 1.0],
            material_buffer,
        })
    }

    /// Créer un matériau avec couleur uniquement
    pub fn with_color(
        device: &wgpu::Device,
        color: [f32; 4],
        label: &str,
    ) -> anyhow::Result<Self> {
        // Créer une texture 1x1 transparente par défaut
        let dummy_texture = Texture::create_dummy(device, label)?;

        let material_uniform = MaterialUniform {
            color,
            use_texture: 0,
            _padding: [0; 7],
        };

        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_material_buffer", label)),
            contents: bytemuck::cast_slice(&[material_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = Self::create_bind_group_layout(device);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{}_bind_group", label)),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&dummy_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: material_buffer.as_entire_binding(),
                },
            ],
        });

        Ok(Self {
            diffuse_texture: None,
            bind_group,
            color,
            material_buffer,
        })
    }

    /// Mettre à jour la couleur
    pub fn update_color(&mut self, queue: &wgpu::Queue, color: [f32; 4]) {
        self.color = color;
        let material_uniform = MaterialUniform {
            color,
            use_texture: if self.diffuse_texture.is_some() { 1 } else { 0 },
            _padding: [0; 7],
        };
        queue.write_buffer(&self.material_buffer, 0, bytemuck::cast_slice(&[material_uniform]));
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}