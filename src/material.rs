use derive_more::Deref;
use serde::Deserialize;

use crate::{light::Lighting, object::Inter, render::Color, scene::Scene};

pub const DEFAULT_MATERIAL: Material = Material::Normal;

pub type MaterialRef = u32;

#[derive(Clone, Debug, Deref, Deserialize)]
pub struct Materials(Vec<Material>);

impl Materials {
    pub fn new(materials: Vec<Material>) -> Self {
        Self(materials)
    }

    pub fn get(&self, material_ref: MaterialRef) -> &Material {
        &self[material_ref as usize]
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Material {
    Simple { color: Color },
    Normal,
}

impl Material {
    pub fn color_at(&self, _scene: &Scene, inter: &Inter, lighting: &Lighting) -> Color {
        match self {
            Material::Simple { color } => color * lighting.force,
            Material::Normal => inter.normal.map(|c| 0.5 * (c + 1.0)),
        }
    }
}
