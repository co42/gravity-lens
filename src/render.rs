use glam::Vec3;
use image::ImageBuffer;
use serde::Deserialize;

use crate::{object::Inter, scene::Scene};

pub fn render(scene: &Scene, out: &Output) -> Vec<Pixel> {
    scene
        .camera
        .project(out)
        .map(|px_ray| {
            let Some(inter) = scene.objects.intersect(&px_ray.ray, out.escape) else {
                return Pixel::NoInter;
            };
            let lighting = scene.lights.lighting(&px_ray.ray, &inter);
            let color = scene.objects.color_at(scene, &inter, &lighting);
            Pixel::Inter { inter, color }
        })
        .collect()
}

#[derive(Clone, Debug, Deserialize)]
pub struct Output {
    pub width: u32,
    pub height: u32,
    pub escape: f32,
}

impl Output {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn save_colors(&self, pixels: &[Pixel], path: impl ToString) {
        let img = ImageBuffer::from_fn(self.width, self.height, |x, y| {
            let i = (x + y * self.width) as usize;
            image::Rgb(
                match pixels[i] {
                    Pixel::NoInter => BLACK,
                    Pixel::Inter { color, .. } => color,
                }
                .map(|c| c * 255.0)
                .as_u8vec3()
                .into(),
            )
        });
        img.save(path.to_string()).unwrap();
    }

    pub fn save_normals(&self, pixels: &[Pixel], path: impl ToString) {
        let img = ImageBuffer::from_fn(self.width, self.height, |x, y| {
            let i = (x + y * self.width) as usize;
            image::Rgb(
                match &pixels[i] {
                    Pixel::NoInter => BLACK,
                    Pixel::Inter { inter, .. } => inter.normal.map(|c| 0.5 * (c + 1.0)),
                }
                .map(|c| c * 255.0)
                .as_u8vec3()
                .into(),
            )
        });
        img.save(path.to_string()).unwrap();
    }
}

#[derive(Clone, Debug)]
pub enum Pixel {
    NoInter,
    Inter { inter: Inter, color: Color },
}

pub type Color = Vec3;

pub const BLACK: Color = Color::ZERO;
