use std::error::Error;
use crate::render_backend::Vertex;

pub struct GlbFile {
    document: gltf::Document,
    buffers: Vec<gltf::buffer::Data>
}

impl GlbFile {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Chargement du fichier GLB: {}", path);

        if !std::path::Path::new(path).exists() {
            return Err(format!("Fichier GLB introuvable: {}", path).into());
        }

        let (document, buffers, _images) = gltf::import(path).unwrap();

        Ok(Self { document, buffers })
    }

    pub fn extract_mesh_data(&self) -> Result<(Box<[Vertex]>, Box<[u16]>), Box<dyn Error>> {
        // Récupère le premier mesh
        let mesh = self.document
            .meshes()
            .next()
            .ok_or("Aucun mesh trouvé dans le fichier")?;

        // Récupère la première primitive du mesh
        let primitive = mesh
            .primitives()
            .next()
            .ok_or("Aucune primitive trouvée")?;

        // Extraction des positions
        let reader = primitive.reader(|buffer| Some(&self.buffers[buffer.index()]));

        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or("Pas de positions trouvées")?
            .collect();

        // Extraction des coordonnées de texture
        let tex_coords: Vec<[f32; 2]> = reader
            .read_tex_coords(0)
            .ok_or("Pas de coordonnées de texture trouvées")?
            .into_f32()
            .collect();

        // Vérification que le nombre de positions et tex_coords correspond
        if positions.len() != tex_coords.len() {
            return Err("Nombre de positions et tex_coords ne correspondent pas".into());
        }

        // Création des vertices
        let vertices: Vec<Vertex> = positions
            .iter()
            .zip(tex_coords.iter())
            .map(|(pos, tex)| Vertex {
                position: *pos,
                tex_coords: *tex,
            })
            .collect();

        // Extraction des indices
        let indices: Vec<u16> = reader
            .read_indices()
            .ok_or("Pas d'indices trouvés")?
            .into_u32()
            .map(|i| i as u16)
            .collect();

        Ok((vertices.into_boxed_slice(), indices.into_boxed_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_backend::glb_loader;
    #[test]
    fn main_test() {
        let model = GlbFile::load("src/model/rocket.glb");
        assert!(model.is_ok());
        let model = model.unwrap();
        let vertices = model.extract_mesh_data().unwrap();
    }
}