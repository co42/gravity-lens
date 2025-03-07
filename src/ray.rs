use glam::Vec3;
use serde::Deserialize;

use crate::render::Output;

#[derive(Clone, Debug, PartialEq)]
pub struct Ray {
    pub pos: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        Self { pos, dir }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.pos + self.dir * t
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Camera {
    pub pos: Vec3,
    pub dir: Vec3,
    pub up: Vec3,
    pub fov: f32,
}

impl Camera {
    /// Sets the camera to look at a specific target position
    pub fn look_at(pos: Vec3, target: Vec3, up: Vec3, fov: f32) -> Self {
        let dir = (target - pos).normalize();
        let right = dir.cross(up).normalize();
        let up = right.cross(dir).normalize();
        Self { pos, dir, up, fov }
    }

    fn right(&self) -> Vec3 {
        self.dir.cross(self.up).normalize()
    }

    pub fn project(&self, output: &Output) -> ProjectIter {
        ProjectIter::new(self.clone(), output.clone())
    }
}

pub struct ProjectIter {
    cam: Camera,
    out: Output,
    screen_down: Vec3,
    screen_right: Vec3,
    x: u32,
    y: u32,
}

impl ProjectIter {
    pub fn new(cam: Camera, out: Output) -> Self {
        let screen_down = -cam.up * cam.fov.tan();
        let screen_right = cam.right() * cam.fov.tan() * out.aspect_ratio();

        Self {
            cam,
            out,
            x: 0,
            y: 0,
            screen_down,
            screen_right,
        }
    }
}

impl Iterator for ProjectIter {
    type Item = PixelRay;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.out.height {
            return None;
        }

        let x = (self.x as f32 + 0.5) / self.out.width as f32;
        let y = (self.y as f32 + 0.5) / self.out.height as f32;

        let dir = self.cam.dir + self.screen_right * (x - 0.5) + self.screen_down * (y - 0.5);
        let dir = dir.normalize();

        let ray_pixel = PixelRay {
            x: self.x,
            y: self.y,
            ray: Ray::new(self.cam.pos, dir),
        };

        self.x += 1;
        if self.x >= self.out.width {
            self.x = 0;
            self.y += 1;
        }

        Some(ray_pixel)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PixelRay {
    pub x: u32,
    pub y: u32,
    pub ray: Ray,
}
