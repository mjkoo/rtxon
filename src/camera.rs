use rand::random;

use crate::types::{Point3, Ray, Scalar, Vector3};

/// Sample a random point in the unit disk via rejection
fn random_in_unit_disk() -> Vector3 {
    let offset = Vector3::new(1.0, 1.0, 0.0);
    let mut p: Vector3;
    while {
        p = 2.0 * Vector3::new(random::<Scalar>(), random::<Scalar>(), 0.0) - offset;
        p.dot(&p) >= 1.0
    } {}

    p
}

/// Adjustable camera for generating eye rays according to given parameters
#[derive(Debug, Clone)]
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vector3,
    vertical: Vector3,
    u: Vector3,
    v: Vector3,
    w: Vector3,
    lens_radius: Scalar,
}

impl Camera {
    /// Create a new camera
    pub fn new(
        origin: Point3,
        lookat: Point3,
        up: Vector3,
        vfov_degrees: Scalar,
        aspect_ratio: Scalar,
        aperture: Scalar,
        focal_length: Scalar,
    ) -> Self {
        let theta = vfov_degrees * std::f32::consts::PI / 180.0;
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

    /// Get a ray from origin intersecting viewing plane at coordinates s and t
    pub fn get_ray(&self, s: Scalar, t: Scalar) -> Ray {
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
