use std::cmp::Ordering;
use std::rc::Rc;

use bvh::ray::Ray;
use nalgebra::{Point3, Vector3};

use crate::materials::Material;
use crate::utils::point_at_parameter;

#[derive(Debug, Clone)]
pub struct HitResult {
    pub t: f32,
    pub p: Point3<f32>,
    pub normal: Vector3<f32>,
    pub material: Rc<dyn Material>,
}

pub trait Shape {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult>;
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    pub material: Rc<dyn Material>,
}

impl Shape for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;

        // This seems wrong in that it doesn't detect which point is closest to ray origin
        let discriminant = b * b - 4.0 * a * c;
        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t = t1.min(t2);

        if discriminant > 0.0 && t > t_min && t < t_max {
            let p = point_at_parameter(&ray, t);
            let normal = (p - self.center) / self.radius;
            Some(HitResult {
                t,
                p,
                normal,
                material: self.material.clone(),
            })
        } else {
            None
        }
    }
}

pub type Scene = Vec<Box<dyn Shape>>;

impl Shape for Scene {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        self.iter()
            .filter_map(|h| h.hit(&ray, t_min, t_max))
            .min_by(|x, y| x.t.partial_cmp(&y.t).unwrap_or(Ordering::Equal))
    }
}
