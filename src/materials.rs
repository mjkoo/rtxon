use rand::random;

use crate::shapes::HitResult;
use crate::types::{Color, Ray, Vector3};

fn random_in_unit_sphere() -> Vector3 {
    let offset = Vector3::new(1.0, 1.0, 1.0);
    let mut p: Vector3;
    while {
        p = 2.0 * Vector3::new(random::<f32>(), random::<f32>(), random::<f32>()) - offset;
        p.magnitude_squared() >= 1.0
    } {}

    p
}

fn reflect(v: Vector3, n: Vector3) -> Vector3 {
    v - 2.0 * v.dot(&n) * n
}

fn refract(v: Vector3, n: Vector3, ni_over_nt: f32) -> Option<Vector3> {
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
    pub attenuation: Color,
}

pub trait Material: Send + Sync + std::fmt::Debug {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay>;
}

#[derive(Debug, Clone)]
pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let _ = ray;
        let target = hit.p.coords + hit.normal + random_in_unit_sphere();
        Some(ScatteredRay {
            ray: Ray::new(hit.p, target - hit.p.coords),
            attenuation: self.albedo,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Metal {
    pub albedo: Color,
    pub roughness: f32,
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let reflected = reflect(ray.direction.normalize(), hit.normal);
        if reflected.dot(&hit.normal) > 0.0 {
            Some(ScatteredRay {
                ray: Ray::new(hit.p, reflected + self.roughness * random_in_unit_sphere()),
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dialectric {
    pub albedo: Color,
    pub ior: f32,
}

impl Material for Dialectric {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
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
                    attenuation: self.albedo,
                });
            }
        }

        Some(ScatteredRay {
            ray: Ray::new(hit.p, reflected),
            attenuation: self.albedo,
        })
    }
}
