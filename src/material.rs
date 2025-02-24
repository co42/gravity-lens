use derive_more::Deref;
use glam::Vec3;
use serde::Deserialize;

use crate::render::Color;

pub type MaterialRef = u32;

#[derive(Clone, Debug, Deref, Deserialize)]
pub struct Materials(Vec<Material>);

impl Materials {
    pub fn color_at(&self, material_ref: MaterialRef, point: Vec3) -> Color {
        self[material_ref as usize].color_at(point)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Material {
    Normal { color: Color },
}

impl Material {
    pub fn color_at(&self, _point: Vec3) -> Color {
        match self {
            Material::Normal { color } => *color,
        }
    }
}
