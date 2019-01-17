use std::f32::consts::PI;

use bvh::ray::Ray;
use nalgebra::{Point3, Vector3};

use crate::utils::random_in_unit_disk;

#[derive(Debug, Clone)]
pub struct Camera {
    origin: Point3<f32>,
    lower_left_corner: Point3<f32>,
    horizontal: Vector3<f32>,
    vertical: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    w: Vector3<f32>,
    lens_radius: f32,
}

impl Camera {
    pub fn new(
        origin: Point3<f32>,
        lookat: Point3<f32>,
        up: Vector3<f32>,
        vfov_degrees: f32,
        aspect_ratio: f32,
        aperture: f32,
        focal_length: f32,
    ) -> Self {
        let theta = vfov_degrees * PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = aspect_ratio * half_height;

        let w = (origin - lookat).normalize();
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);

        Self {
            origin,
            lower_left_corner: origin
                - half_width * focal_length * u
                - half_height * focal_length * v
                - focal_length * w,
            horizontal: 2.0 * half_width * focal_length * u,
            vertical: 2.0 * half_height * focal_length * v,
            u,
            v,
            w,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = rd.x * self.u + rd.y * self.v;
        Ray::new(
            self.origin + offset,
            self.lower_left_corner.coords + s * self.horizontal + t * self.vertical
                - self.origin.coords
                - offset,
        )
    }
}
