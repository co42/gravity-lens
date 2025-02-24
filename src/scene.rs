use serde::Deserialize;

use crate::{light::Lights, material::Materials, object::Objects, ray::Camera};

#[derive(Clone, Debug, Deserialize)]
pub struct Scene {
    pub camera: Camera,
    pub lights: Lights,
    pub materials: Materials,
    pub objects: Objects,
}
