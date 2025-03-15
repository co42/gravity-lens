use std::{cell::RefCell, f32::consts::PI, fs::File, io::BufReader};

use glam::Vec3;
use image::{DynamicImage, ImageFormat, Rgb32FImage};
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
    Hdri(Hdri),
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
            Light::Hdri(hdri) => {
                // TODO: Do multiple samples to reduce noise
                hdri.sample(inter.normal)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Hdri {
    pub path: String,
    #[serde(default, skip)]
    pub image: RefCell<Option<Rgb32FImage>>,
}

impl Hdri {
    pub fn load(&self) {
        let file = File::open(&self.path).expect("Open HDRI");
        let buf = BufReader::new(file);
        let image = image::load(buf, ImageFormat::Hdr).expect("Load HDRI");
        let image = match image {
            DynamicImage::ImageRgb32F(image) => image,
            _ => panic!("HDRI format should be Rgb32F"),
        };
        *self.image.borrow_mut() = Some(image);
    }

    pub fn sample(&self, dir: Vec3) -> Color {
        // Convert from cartesian coordinates to spherical coordinates
        // θ (theta) is the polar angle from the y-axis (up)
        // φ (phi) is the azimuthal angle in the xz plane from the x-axis
        let theta = dir.y.acos(); // Range [0, π]
        let phi = dir.z.atan2(dir.x) + PI; // Range [0, 2π]

        // Map spherical coordinates to UV coordinates
        // U ranges from 0 to 1 based on phi (longitude)
        // V ranges from 0 to 1 based on theta (latitude)
        let u = phi / (2.0 * PI); // Convert [0, 2π] to [0, 1]
        let v = theta / PI; // Convert [0, π] to [0, 1]

        // TODO: Load on startup and remove RefCell
        if self.image.borrow().is_none() {
            self.load();
        }
        let i = self.image.borrow();
        let image = i.as_ref().expect("HDRI image not loaded");
        let (width, height) = image.dimensions();
        let x = (u * width as f32) as u32;
        let y = ((1.0 - v) * height as f32) as u32;
        let pixel = image.get_pixel(x, y);
        Color::new(pixel[0], pixel[1], pixel[2])
    }
}
