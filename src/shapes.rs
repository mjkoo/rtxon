use std::cmp::Ordering;
use std::sync::Arc;

use crate::materials::Material;
use crate::types::{Point3, Ray, Scalar, Vector3};

/// Result of ray intersection with a shape
#[derive(Debug, Clone)]
pub struct HitResult {
    pub t: Scalar,
    pub p: Point3,
    pub normal: Vector3,
    pub material: Arc<dyn Material>,
}

/// Shape defines objects intersectable by rays
pub trait Shape: Send + Sync {
    /// Does an incoming ray intersect this shape
    fn hit(&self, ray: &Ray, t_min: Scalar, t_max: Scalar) -> Option<HitResult>;
}

/// Spherical shape
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point3,
    pub radius: Scalar,
    pub material: Arc<dyn Material>,
}

impl Shape for Sphere {
    /// Does an incoming ray intersect this shape
    fn hit(&self, ray: &Ray, t_min: Scalar, t_max: Scalar) -> Option<HitResult> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        let t = if discriminant > 0.0 && t1 < t2 && t1 > t_min && t1 < t_max {
            Some(t1)
        } else if discriminant > 0.0 && t2 < t1 && t2 > t_min && t2 < t_max {
            Some(t2)
        } else {
            None
        };

        t.and_then(|t| {
            let p = ray.at(t);
            let normal = (p - self.center) / self.radius;
            Some(HitResult {
                t,
                p,
                normal,
                material: self.material.clone(),
            })
        })
    }
}

/// A collection of other shapes, itself intersectable
pub type Scene = Vec<Arc<dyn Shape>>;

impl Shape for Scene {
    /// Does an incoming ray intersect this shape
    fn hit(&self, ray: &Ray, t_min: Scalar, t_max: Scalar) -> Option<HitResult> {
        // TODO: Use an acceleration structure such as a BVH to optimize this
        self.iter()
            .filter_map(|h| h.hit(&ray, t_min, t_max))
            .min_by(|x, y| x.t.partial_cmp(&y.t).unwrap_or(Ordering::Equal))
    }
}
