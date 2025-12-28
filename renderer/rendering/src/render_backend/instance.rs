use cgmath::{Matrix4, Quaternion, Vector3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Instance {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Instance {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }

    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position)
                * Matrix4::from(self.rotation))
                .into(),
        }
    }
}

pub struct InstanceBuffer {
    buffer: wgpu::Buffer,
    instances: Vec<Instance>,
}

impl InstanceBuffer {
    pub fn new(device: &wgpu::Device, instances: Vec<Instance>) -> Self {
        let instance_data: Vec<InstanceRaw> =
            instances.iter().map(Instance::to_raw).collect();

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self { buffer, instances }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        let instance_data: Vec<InstanceRaw> =
            self.instances.iter().map(Instance::to_raw).collect();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&instance_data));
    }

    pub fn update_instance(&mut self, index: usize, position: Vector3<f32>) {
        if index < self.instances.len() {
            self.instances[index].position = position;
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        InstanceRaw::desc()
    }
}