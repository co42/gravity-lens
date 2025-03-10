use glam::Vec3;
use serde::Deserialize;

use crate::{object::Inter, ray::Ray, render::Color};

#[derive(Clone, Debug)]
pub struct Lighting {
    pub force: Vec3,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Lights(Vec<Light>);

impl Lights {
    pub fn new(lights: Vec<Light>) -> Self {
        Self(lights)
    }

    pub fn lighting(&self, ray: &Ray, inter: &Inter) -> Lighting {
        let force = self.0.iter().map(|light| light.lighting(ray, inter)).sum();
        Lighting { force }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Light {
    Ambient { intensity: Vec3 },
    Point { pos: Vec3, intensity: Vec3 },
    Directional { dir: Vec3, intensity: Vec3 },
}

impl Light {
    pub fn lighting(&self, ray: &Ray, inter: &Inter) -> Color {
        self.diffuse(ray, inter)
    }

    fn diffuse(&self, ray: &Ray, inter: &Inter) -> Color {
        match self {
            Light::Ambient { intensity } => *intensity,
            Light::Point { pos, intensity } => {
                let light_dir = (*pos - ray.at(inter.t)).normalize();
                let dot = inter.normal.dot(light_dir).clamp(0.0, 1.0);
                *intensity * dot
            }
            Light::Directional { dir, intensity } => {
                let dot = inter.normal.dot(*dir).clamp(0.0, 1.0);
                *intensity * dot
            }
        }
    }
}
