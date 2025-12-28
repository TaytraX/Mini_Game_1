use std::sync::Arc;
use std::time::Duration;
use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::camera::{Camera, CameraController, CameraUniform, Projection};
use crate::render_backend::context::WgpuContext;
use crate::render_backend::Material;
use crate::render_backend::Mesh;
use crate::render_backend::InstanceBuffer;
use crate::render_backend::RenderPipelineBuilder;
use crate::render_backend::{Scene, SceneObject};
use crate::texture::Texture;

pub struct State {
    pub window: Arc<Window>,
    context: WgpuContext,
    render_pipeline: wgpu::RenderPipeline,
    camera: Camera,
    projection: Projection,
    pub camera_controller: CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform: CameraUniform,
    depth_texture: Texture,
    scene: Scene,
    chunk_renderer: crate::chunk_renderer::ChunkRenderer,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let mut context = WgpuContext::new(window.clone()).await?;

        let size = window.inner_size();
        context.resize(size.width, size.height);

        // Camera setup
        let camera = Camera::new(
            (0.0, 5.0, 10.0),
            cgmath::Deg(-90.0),
            cgmath::Deg(-20.0),
        );
        let projection = Projection::new(
            context.config.width,
            context.config.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout = Self::create_camera_bind_group_layout(&context.device);

        let camera_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Pipeline
        let pipeline_builder = RenderPipelineBuilder::new(context.device.clone());
        let render_pipeline = pipeline_builder.build(context.format(), &camera_bind_group_layout);

        // Depth texture
        let depth_texture = Texture::create_depth_texture(
            &context.device,
            &context.config,
            Some("Depth Texture"),
        );

        // Initialiser le gestionnaire de types de blocs et le renderer
        let block_manager = crate::block_types::BlockTypeManager::new()?;
        let chunk_renderer = crate::chunk_renderer::ChunkRenderer::new(&context.device, block_manager);

        // Créer la scène vide
        let mut scene = Scene::new();

        // Pour l'instant, créer un chunk de test
        // Ce chunk sera remplacé par les données venant de Java via JNI
        let mut test_chunk = vec![0.0f32; 32 * 32 * 32];

        // Ajouter quelques blocs de test
        for x in 0..10 {
            for z in 0..10 {
                // Sol en pierre (type 2)
                test_chunk[0 * (32 * 32) + z * 32 + x] = 2.0;
                // Quelques blocs de terre (type 1)
                if x % 2 == 0 && z % 2 == 0 {
                    test_chunk[1 * (32 * 32) + z * 32 + x] = 1.0;
                }
            }
        }

        // Générer la scène à partir du chunk
        chunk_renderer.generate_scene(&context.device, &test_chunk, &mut scene)?;

        Ok(Self {
            window,
            context,
            render_pipeline,
            camera,
            projection,
            camera_controller: CameraController::new(4.0, 0.4),
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            depth_texture,
            scene,
            chunk_renderer,
        })
    }

    fn create_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
        self.projection.resize(width, height);
        self.depth_texture = Texture::create_depth_texture(
            &self.context.device,
            &self.context.config,
            Some("Depth Texture"),
        );
    }

    pub fn update(&mut self, dt: Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        self.context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Ne pas mettre à jour le chunk ici - uniquement via update_chunk_from_java()
    }

    pub fn update_instance(&mut self, pos: (f32, f32, f32)) {
        if let Some(object) = self.scene.objects_mut().get_mut(0) {
            object.instance_buffer_mut().update_instance(1, pos.into());
            object.instance_buffer_mut().update(&self.context.queue);
        }
    }

    /// Changer la couleur d'un objet de la scène
    pub fn update_material_color(&mut self, object_index: usize, color: [f32; 4]) {
        if let Some(object) = self.scene.objects_mut().get_mut(object_index) {
            object.material_mut().update_color(&self.context.queue, color);
        }
    }

    /// Mettre à jour le chunk avec de nouvelles données (appelé depuis Java)
    pub fn update_chunk_from_java(&mut self) -> anyhow::Result<()> {
        let chunk_data = crate::jni_interface::get_chunk_data();
        self.update_chunk(&chunk_data)
    }

    /// Mettre à jour le chunk avec de nouvelles données
    fn update_chunk(&mut self, chunk_data: &[f32]) -> anyhow::Result<()> {
        // Vider la scène actuelle
        self.scene = Scene::new();

        // Régénérer la scène avec les nouvelles données
        self.chunk_renderer.generate_scene(
            &self.context.device,
            chunk_data,
            &mut self.scene,
        )?;

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if !self.context.is_configured() {
            return Ok(());
        }

        let output = self.context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.75,
                            g: 0.5,
                            b: 0.25,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            // Render all objects in scene
            for object in self.scene.objects() {
                render_pass.set_bind_group(0, object.material().bind_group(), &[]);
                render_pass.set_vertex_buffer(0, object.mesh().vertex_buffer().slice(..));
                render_pass.set_vertex_buffer(1, object.instance_buffer().buffer().slice(..));
                render_pass.set_index_buffer(
                    object.mesh().index_buffer().slice(..),
                    wgpu::IndexFormat::Uint16,
                );

                render_pass.draw_indexed(
                    0..object.mesh().num_indices(),
                    0,
                    0..object.instance_buffer().len() as u32,
                );
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: KeyCode, pressed: bool) {
        self.camera_controller.handle_key(key, pressed);
        if matches!((key, pressed), (KeyCode::Escape, true)) {
            event_loop.exit();
        }
    }
}