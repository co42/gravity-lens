use glam::Vec3;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Attractors(Vec<Attractor>);

impl Attractors {
    pub fn attract(&self, point: Vec3) -> Vec3 {
        self.0.iter().map(|a| a.attract(point)).sum()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Attractor {
    pub pos: Vec3,
    pub mass: f32,
}

impl Attractor {
    pub fn attract(&self, point: Vec3) -> Vec3 {
        let dir = self.pos - point;
        let force = self.mass / dir.length_squared();
        dir.normalize() * force
    }
}
