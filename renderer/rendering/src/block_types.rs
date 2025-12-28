use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct BlockColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl BlockColor {
    pub fn to_rgba(&self) -> [f32; 4] {
        [self.r, self.g, self.b, 1.0]
    }
}

/// Gère les types de blocs et leurs couleurs
pub struct BlockTypeManager {
    block_key: HashMap<u32, String>,
    block_colors: HashMap<String, BlockColor>,
}

impl BlockTypeManager {
    pub fn new() -> Result<Self> {
        // Charger block_key.json
        let block_key_json = include_str!("../block_key.json");
        let block_key: HashMap<String, String> = serde_json::from_str(block_key_json)?;

        // Convertir les clés String en u32
        let block_key: HashMap<u32, String> = block_key
            .into_iter()
            .filter_map(|(k, v)| k.parse::<u32>().ok().map(|id| (id, v)))
            .collect();

        // Charger block_color.json
        let block_color_json = include_str!("../block_color.json");
        let block_colors: HashMap<String, Vec<f32>> = serde_json::from_str(block_color_json)?;

        // Convertir en BlockColor
        let block_colors: HashMap<String, BlockColor> = block_colors
            .into_iter()
            .map(|(name, rgb)| {
                (name, BlockColor {
                    r: rgb.get(0).copied().unwrap_or(1.0),
                    g: rgb.get(1).copied().unwrap_or(1.0),
                    b: rgb.get(2).copied().unwrap_or(1.0),
                })
            })
            .collect();

        Ok(Self {
            block_key,
            block_colors,
        })
    }

    /// Obtenir la couleur d'un bloc par son ID
    pub fn get_color(&self, block_id: u32) -> Option<[f32; 4]> {
        let block_name = self.block_key.get(&block_id)?;
        let color = self.block_colors.get(block_name)?;
        Some(color.to_rgba())
    }

    /// Obtenir le nom d'un bloc par son ID
    pub fn get_name(&self, block_id: u32) -> Option<&str> {
        self.block_key.get(&block_id).map(|s| s.as_str())
    }
}

impl Default for BlockTypeManager {
    fn default() -> Self {
        Self::new().expect("Failed to load block type configuration")
    }
}