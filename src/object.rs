use glam::Vec3;
use serde::Deserialize;

use crate::{
    light::Lighting,
    material::{MaterialRef, DEFAULT_MATERIAL},
    ray::Ray,
    render::Color,
    scene::Scene,
};

#[derive(Clone, Debug)]
pub struct Inter {
    pub object_ref: ObjectRef,
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

pub type ObjectRef = u32;

#[derive(Clone, Debug, Deserialize)]
pub struct Objects(Vec<Object>);

impl Objects {
    pub fn new(objects: Vec<Object>) -> Self {
        Self(objects)
    }

    pub fn get(&self, object_ref: ObjectRef) -> &Object {
        &self.0[object_ref as usize]
    }

    pub fn intersect(&self, ray: &Ray, max_t: f32) -> Option<Inter> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                object.intersect_in(ray, max_t).map(|t| {
                    let point = ray.at(t);
                    Inter {
                        object_ref: index as ObjectRef,
                        t,
                        point,
                        normal: object.normal_at(point),
                    }
                })
            })
            .min_by(|l, r| l.t.partial_cmp(&r.t).expect("Floats should be comparable"))
    }

    pub fn color_at(&self, scene: &Scene, inter: &Inter, lighting: &Lighting) -> Color {
        scene
            .objects
            .get(inter.object_ref)
            .color_at(scene, inter, lighting)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Object {
    pub material_ref: Option<MaterialRef>,
    #[serde(flatten)]
    pub shape: Shape,
}

impl Object {
    pub fn new(material_ref: Option<MaterialRef>, shape: Shape) -> Self {
        Self {
            material_ref,
            shape,
        }
    }

    pub fn intersect_in(&self, ray: &Ray, max_t: f32) -> Option<f32> {
        self.shape.intersect_in(ray, max_t)
    }

    pub fn normal_at(&self, point: Vec3) -> Vec3 {
        self.shape.normal_at(point)
    }

    pub fn color_at(&self, scene: &Scene, inter: &Inter, lighting: &Lighting) -> Color {
        self.shape
            .color_at(scene, inter, lighting)
            .or_else(|| {
                self.material_ref.map(|material_ref| {
                    scene
                        .materials
                        .get(material_ref)
                        .color_at(scene, &inter, lighting)
                })
            })
            .unwrap_or_else(|| DEFAULT_MATERIAL.color_at(scene, &inter, lighting))
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Shape {
    Sphere(Sphere),
    MetaBalls(MetaBalls),
}

impl Shape {
    pub fn intersect_in(&self, ray: &Ray, max_t: f32) -> Option<f32> {
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

    pub fn color_at(&self, scene: &Scene, inter: &Inter, lighting: &Lighting) -> Option<Color> {
        match self {
            Shape::Sphere(_) => None,
            Shape::MetaBalls(meta_balls) => meta_balls.color_at(scene, inter, lighting),
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
            if (force - self.threshold).abs() < 0.001 {
                return Some(t);
            }

            let point_dist = power / force;
            let threshold_dist = power / self.threshold;
            t += point_dist - threshold_dist;
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

    fn color_at(&self, scene: &Scene, inter: &Inter, lighting: &Lighting) -> Option<Color> {
        self.balls
            .iter()
            .map(|ball| ball.color_at(scene, inter, lighting))
            .reduce(|l, r| match (l, r) {
                (Some(l), Some(r)) => Some(l + r),
                (l, r) => l.or(r),
            })
            .flatten()
            .map(|color| color / self.force_at(inter.point))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetaBall {
    pub material_ref: Option<MaterialRef>,
    pub center: Vec3,
    pub power: f32,
}

impl MetaBall {
    fn strength_at(&self, point: Vec3) -> f32 {
        let diff = point - self.center;
        self.power / diff.length()
    }

    fn force_at(&self, point: Vec3) -> Vec3 {
        let diff = self.center - point;
        let strength = self.power / diff.length();
        strength * diff.normalize()
    }

    fn color_at(&self, scene: &Scene, inter: &Inter, lighting: &Lighting) -> Option<Color> {
        self.material_ref.map(|material_ref| {
            scene
                .materials
                .get(material_ref)
                .color_at(scene, inter, lighting)
                * self.strength_at(inter.point)
        })
    }
}
