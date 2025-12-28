use crate::render_backend::mesh::Mesh;
use crate::render_backend::material::Material;
use crate::render_backend::instance::{Instance, InstanceBuffer};
use cgmath::{Quaternion, Vector3, Deg, InnerSpace, Rotation3};

pub struct SceneObject {
    mesh: Mesh,
    material: Material,
    instance_buffer: InstanceBuffer,
}

impl SceneObject {
    pub fn new(
        mesh: Mesh,
        material: Material,
        instance_buffer: InstanceBuffer,
    ) -> Self {
        Self {
            mesh,
            material,
            instance_buffer,
        }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    /// Accès mutable au matériau pour changer la couleur
    pub fn material_mut(&mut self) -> &mut Material {
        &mut self.material
    }

    pub fn instance_buffer(&self) -> &InstanceBuffer {
        &self.instance_buffer
    }

    pub fn instance_buffer_mut(&mut self) -> &mut InstanceBuffer {
        &mut self.instance_buffer
    }
}

pub struct Scene {
    objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: SceneObject) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &[SceneObject] {
        &self.objects
    }

    pub fn objects_mut(&mut self) -> &mut [SceneObject] {
        &mut self.objects
    }

    pub fn create_grid_instances(rows: u32, cols: u32) -> Vec<Instance> {
        let displacement = Vector3::new(
            rows as f32 * 0.5,
            0.0,
            cols as f32 * 0.5,
        );

        (0..rows)
            .flat_map(|z| {
                (0..cols).map(move |x| {
                    let position = Vector3 {
                        x: x as f32 * 4.0,
                        y: 0.0,
                        z: z as f32 * 4.0,
                    } - displacement;

                    let rotation = if position.magnitude() < 0.001 {
                        Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
                    } else {
                        Quaternion::from_axis_angle(position.normalize(), Deg(1.0))
                    };

                    Instance::new(position, rotation)
                })
            })
            .collect()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}