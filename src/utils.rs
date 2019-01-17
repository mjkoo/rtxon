use bvh::ray::Ray;
use image::Rgb;
use nalgebra::{Point3, Vector3};
use rand::random;

pub fn f32_to_u8(f: f32) -> u8 {
    (f * 255.99) as u8
}

pub fn u8_to_f32(b: u8) -> f32 {
    (b as f32) / 255.0
}

// TODO: Implement From for these conversions
pub fn vector3_to_color(v: Vector3<f32>) -> Rgb<u8> {
    Rgb([f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z)])
}

pub fn color_to_vector3(color: Rgb<u8>) -> Vector3<f32> {
    Vector3::new(
        u8_to_f32(color[0]),
        u8_to_f32(color[1]),
        u8_to_f32(color[2]),
    )
}

pub fn point_at_parameter(ray: &Ray, t: f32) -> Point3<f32> {
    ray.origin + t * ray.direction.normalize()
}

pub fn random_in_unit_sphere() -> Vector3<f32> {
    let mut p: Vector3<f32>;
    while {
        p = 2.0 * Vector3::new(random::<f32>(), random::<f32>(), random::<f32>());
        p.magnitude_squared() >= 1.0
    } {}

    p
}

pub fn random_in_unit_disk() -> Vector3<f32> {
    let mut p: Vector3<f32>;
    while {
        p = 2.0 * Vector3::new(random::<f32>(), random::<f32>(), 0.0) - Vector3::new(1.0, 1.0, 0.0);
        p.dot(&p) >= 1.0
    } {}

    p
}
