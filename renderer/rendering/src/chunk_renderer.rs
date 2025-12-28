use crate::render_backend::{Material, Mesh, Scene, SceneObject, InstanceBuffer};
use crate::render_backend::mesh::Vertex;
use crate::render_backend::instance::Instance;
use crate::block_types::BlockTypeManager;
use cgmath::{Vector3, Quaternion, Zero};
use std::collections::HashMap;

const CHUNK_SIZE: usize = 32;

pub struct ChunkRenderer {
    block_manager: BlockTypeManager,
    cube_mesh: Mesh,
}

impl ChunkRenderer {
    pub fn new(device: &wgpu::Device, block_manager: BlockTypeManager) -> Self {
        let cube_mesh = Self::create_cube_mesh(device);

        Self {
            block_manager,
            cube_mesh,
        }
    }

    /// Créer un mesh de cube unitaire (1x1x1)
    fn create_cube_mesh(device: &wgpu::Device) -> Mesh {
        #[rustfmt::skip]
        const VERTICES: &[Vertex] = &[
            // Face avant
            Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 0.0] },

            // Face arrière
            Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [0.0, 0.0] },

            // Face gauche
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0] },

            // Face droite
            Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [0.0, 0.0] },

            // Face haut
            Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0] },

            // Face bas
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 0.0] },
        ];

        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0,  1,  2,  2,  3,  0,  // Avant
            4,  5,  6,  6,  7,  4,  // Arrière
            8,  9, 10, 10, 11,  8,  // Gauche
            12, 13, 14, 14, 15, 12, // Droite
            16, 17, 18, 18, 19, 16, // Haut
            20, 21, 22, 22, 23, 20, // Bas
        ];

        Mesh::from_vertices(device, VERTICES, INDICES)
    }

    /// Convertir les coordonnées 3D en index 1D
    fn coord_to_index(x: usize, y: usize, z: usize) -> usize {
        y * (CHUNK_SIZE * CHUNK_SIZE) + z * CHUNK_SIZE + x
    }

    /// Vérifier si un bloc existe aux coordonnées données
    fn is_block_solid(chunk_data: &[f32], x: i32, y: i32, z: i32) -> bool {
        if x < 0 || y < 0 || z < 0
            || x >= CHUNK_SIZE as i32
            || y >= CHUNK_SIZE as i32
            || z >= CHUNK_SIZE as i32 {
            return false;
        }

        let idx = Self::coord_to_index(x as usize, y as usize, z as usize);
        chunk_data.get(idx).copied().unwrap_or(0.0) > 0.0
    }

    /// Vérifier si une face est visible (optimisation)
    fn is_face_visible(chunk_data: &[f32], x: i32, y: i32, z: i32, dx: i32, dy: i32, dz: i32) -> bool {
        !Self::is_block_solid(chunk_data, x + dx, y + dy, z + dz)
    }

    /// Générer la scène à partir des données du chunk
    pub fn generate_scene(
        &self,
        device: &wgpu::Device,
        chunk_data: &[f32],
        scene: &mut Scene,
    ) -> anyhow::Result<()> {
        // Regrouper les blocs par type pour optimiser le rendu
        let mut blocks_by_type: HashMap<u32, Vec<Vector3<f32>>> = HashMap::new();

        // Parcourir tous les blocs du chunk
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let idx = Self::coord_to_index(x, y, z);
                    let block_type = chunk_data.get(idx).copied().unwrap_or(0.0) as u32;

                    // Ignorer les blocs vides (type 0)
                    if block_type == 0 {
                        continue;
                    }

                    // Vérifier si au moins une face est visible (optimisation)
                    let has_visible_face =
                        Self::is_face_visible(chunk_data, x as i32, y as i32, z as i32, 1, 0, 0) ||
                            Self::is_face_visible(chunk_data, x as i32, y as i32, z as i32, -1, 0, 0) ||
                            Self::is_face_visible(chunk_data, x as i32, y as i32, z as i32, 0, 1, 0) ||
                            Self::is_face_visible(chunk_data, x as i32, y as i32, z as i32, 0, -1, 0) ||
                            Self::is_face_visible(chunk_data, x as i32, y as i32, z as i32, 0, 0, 1) ||
                            Self::is_face_visible(chunk_data, x as i32, y as i32, z as i32, 0, 0, -1);

                    if has_visible_face {
                        let position = Vector3::new(x as f32, y as f32, z as f32);
                        blocks_by_type
                            .entry(block_type)
                            .or_insert_with(Vec::new)
                            .push(position);
                    }
                }
            }
        }

        // Créer un objet de scène pour chaque type de bloc
        for (block_type, positions) in blocks_by_type {
            // Obtenir la couleur du bloc
            let color = self.block_manager
                .get_color(block_type)
                .unwrap_or([1.0, 0.0, 1.0, 1.0]); // Magenta par défaut si non trouvé

            // Créer le matériau
            let block_name = self.block_manager
                .get_name(block_type)
                .unwrap_or("unknown");
            let material = Material::with_color(
                device,
                color,
                &format!("block_{}", block_name),
            )?;

            // Créer les instances pour tous les blocs de ce type
            let instances: Vec<Instance> = positions
                .iter()
                .map(|pos| {
                    Instance::new(*pos, Quaternion::zero())
                })
                .collect();

            let instance_buffer = InstanceBuffer::new(device, instances);

            // Ajouter à la scène
            scene.add_object(SceneObject::new(
                self.cube_mesh.clone(),
                material,
                instance_buffer,
            ));
        }

        Ok(())
    }
}

/// Cloner Mesh pour pouvoir réutiliser le même mesh pour plusieurs objets
impl Clone for Mesh {
    fn clone(&self) -> Self {
        Self {
            vertex_buffer: self.vertex_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            num_indices: self.num_indices,
        }
    }
}