use bvh::ray::Ray;
use image::Rgb;
use nalgebra::Vector3;
use rand::random;

use crate::shapes::HitResult;
use crate::utils::{color_to_vector3, random_in_unit_sphere};

fn reflect(v: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * v.dot(&n) * n
}

fn refract(v: Vector3<f32>, n: Vector3<f32>, ni_over_nt: f32) -> Option<Vector3<f32>> {
    let uv = v.normalize();
    let dt = uv.dot(&n);
    let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);

    if discriminant > 0.0 {
        let refracted = ni_over_nt * (uv - n * dt) - n * discriminant.sqrt();
        Some(refracted)
    } else {
        None
    }
}

fn schlick(cosine: f32, ior: f32) -> f32 {
    let sqrt_r0 = (1.0 - ior) / (1.0 + ior);
    let r0 = sqrt_r0 * sqrt_r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

pub struct ScatteredRay {
    pub ray: Ray,
    pub attenuation: Vector3<f32>,
}

pub trait Material: std::fmt::Debug {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay>;
}

#[derive(Debug, Clone)]
pub struct Lambertian {
    pub albedo: Rgb<u8>,
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let _ = ray;
        let target = hit.p.coords + hit.normal + random_in_unit_sphere();
        Some(ScatteredRay {
            ray: Ray::new(hit.p, target - hit.p.coords),
            attenuation: color_to_vector3(self.albedo),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Metal {
    pub albedo: Rgb<u8>,
    pub roughness: f32,
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let reflected = reflect(ray.direction.normalize(), hit.normal);
        if reflected.dot(&hit.normal) > 0.0 {
            Some(ScatteredRay {
                ray: Ray::new(hit.p, reflected + self.roughness * random_in_unit_sphere()),
                attenuation: color_to_vector3(self.albedo),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dialectric {
    pub albedo: Rgb<u8>,
    pub ior: f32,
}

impl Material for Dialectric {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let attenuation = color_to_vector3(self.albedo);

        let reflected = reflect(ray.direction, hit.normal);
        let dot = ray.direction.dot(&hit.normal) / ray.direction.magnitude();

        let (outward_normal, ni_over_nt, cosine) = if dot > 0.0 {
            (
                -hit.normal,
                self.ior,
                (1.0 - self.ior * self.ior * (1.0 - dot * dot)).sqrt(),
            )
        } else {
            (hit.normal, 1.0 / self.ior, -dot)
        };

        if let Some(refracted) = refract(ray.direction, outward_normal, ni_over_nt) {
            if random::<f32>() >= schlick(cosine, self.ior) {
                return Some(ScatteredRay {
                    ray: Ray::new(hit.p, refracted),
                    attenuation,
                });
            }
        }

        Some(ScatteredRay {
            ray: Ray::new(hit.p, reflected),
            attenuation,
        })
    }
}
