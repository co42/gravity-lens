use glam::Vec3;
use serde::Deserialize;

use crate::{material::MaterialRef, ray::Ray};

#[derive(Clone, Debug)]
pub struct Inter {
    pub t: f32,
    pub normal: Vec3,
    pub material_ref: MaterialRef,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Objects(Vec<Object>);

impl Objects {
    pub fn intersect(&self, ray: &Ray, max_t: f32) -> Option<Inter> {
        self.0
            .iter()
            .filter_map(|object| object.intersect(ray, max_t))
            .min_by(|l, r| l.t.partial_cmp(&r.t).expect("Floats should be comparable"))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Object {
    pub material_ref: MaterialRef,
    #[serde(flatten)]
    pub shape: Shape,
}

impl Object {
    pub fn intersect(&self, ray: &Ray, max_t: f32) -> Option<Inter> {
        self.shape.intersect(ray, max_t).map(|t| Inter {
            t,
            normal: self.shape.normal_at(ray.at(t)),
            material_ref: self.material_ref,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Shape {
    Sphere(Sphere),
    MetaBalls(MetaBalls),
}

impl Shape {
    pub fn intersect(&self, ray: &Ray, max_t: f32) -> Option<f32> {
        match self {
            Shape::Sphere(sphere) => sphere.intersect_in(ray, max_t),
            Shape::MetaBalls(meta_balls) => meta_balls.intersect_in(ray, max_t),
        }
    }

    pub fn normal_at(&self, point: Vec3) -> Vec3 {
        match self {
            Shape::Sphere(sphere) => sphere.normal_at(point),
            Shape::MetaBalls(meta_balls) => meta_balls.normal_at(point),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    fn intersect_in(&self, ray: &Ray, max_t: f32) -> Option<f32> {
        let oc = ray.pos - self.center;
        let a = ray.dir.length_squared();
        let b = 2.0 * oc.dot(ray.dir);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t > 0.0 {
            (t < max_t).then_some(t)
        } else {
            let t = (-b + discriminant.sqrt()) / (2.0 * a);
            (t < max_t).then_some(t)
        }
    }

    fn normal_at(&self, point: Vec3) -> Vec3 {
        (point - self.center).normalize()
    }
}

/// https://en.wikipedia.org/wiki/Metaballs
/// Force function is inverse distance to the center of each ball times their respective power
/// TODO: Handle ray coming from inside
#[derive(Clone, Debug, Deserialize)]
pub struct MetaBalls {
    pub balls: Vec<MetaBall>,
    pub threshold: f32,
}

impl MetaBalls {
    fn intersect_in(&self, ray: &Ray, max_t: f32) -> Option<f32> {
        let power = self.balls.iter().map(|ball| ball.power).sum::<f32>();
        let mut t = 0.0;
        while t < max_t {
            let point = ray.pos + t * ray.dir;
            let force = self.force_at(point);
            if force >= self.threshold {
                return Some(t);
            }

            let point_dist = power / force;
            let threshold_dist = power / self.threshold;
            t += (point_dist - threshold_dist).max(0.01);
        }
        None
    }

    fn normal_at(&self, point: Vec3) -> Vec3 {
        -self
            .balls
            .iter()
            .map(|ball| ball.force_at(point))
            .sum::<Vec3>()
            .normalize()
    }

    fn force_at(&self, point: Vec3) -> f32 {
        self.balls.iter().map(|ball| ball.strength_at(point)).sum()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetaBall {
    pub center: Vec3,
    pub power: f32,
}

impl MetaBall {
    fn force_at(&self, point: Vec3) -> Vec3 {
        let diff = self.center - point;
        let strength = self.power / diff.length();
        strength * diff.normalize()
    }

    fn strength_at(&self, point: Vec3) -> f32 {
        let diff = point - self.center;
        self.power / diff.length()
    }
}
