use std::cmp::Ordering;
use std::rc::Rc;

use bvh::ray::Ray;
use clap::{value_t_or_exit, App, Arg};
use failure::Error;
use image::{ImageBuffer, Rgb};
use log::info;
use nalgebra::{Point3, Vector3};
use rand::random;

fn f32_to_u8(f: f32) -> u8 {
    (f * 255.99) as u8
}

fn u8_to_f32(b: u8) -> f32 {
    (b as f32) / 255.0
}

// TODO: Implement From for these conversions
fn vector3_to_color(v: Vector3<f32>) -> Rgb<u8> {
    Rgb([f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z)])
}

fn color_to_vector3(color: Rgb<u8>) -> Vector3<f32> {
    Vector3::new(
        u8_to_f32(color[0]),
        u8_to_f32(color[1]),
        u8_to_f32(color[2]),
    )
}

fn point_at_parameter(ray: &Ray, t: f32) -> Point3<f32> {
    ray.origin + t * ray.direction.normalize()
}

fn random_in_unit_square() -> Vector3<f32> {
    let mut p: Vector3<f32>;
    while {
        p = 2.0 * Vector3::new(random::<f32>(), random::<f32>(), random::<f32>());
        p.magnitude_squared() >= 1.0
    } {}

    p
}

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

#[derive(Debug, Clone)]
struct Camera {
    origin: Point3<f32>,
    lower_left_corner: Point3<f32>,
    horizontal: Vector3<f32>,
    vertical: Vector3<f32>,
}

impl Camera {
    fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner.coords + u * self.horizontal + v * self.vertical,
        )
    }
}

#[derive(Debug, Clone)]
struct HitResult {
    t: f32,
    p: Point3<f32>,
    normal: Vector3<f32>,
    material: Rc<dyn Material>,
}

trait Hit {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult>;
}

#[derive(Debug, Clone)]
struct Sphere {
    center: Point3<f32>,
    radius: f32,
    material: Rc<dyn Material>,
}

impl Hit for Sphere {
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

impl Hit for Vec<Box<dyn Hit>> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult> {
        self.iter()
            .filter_map(|h| h.hit(&ray, t_min, t_max))
            .min_by(|x, y| x.t.partial_cmp(&y.t).unwrap_or(Ordering::Equal))
    }
}

struct ScatteredRay {
    ray: Ray,
    attenuation: Vector3<f32>,
}

trait Material: std::fmt::Debug {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay>;
}

#[derive(Debug, Clone)]
struct Lambertian {
    albedo: Rgb<u8>,
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let _ = ray;
        let target = hit.p.coords + hit.normal + random_in_unit_square();
        Some(ScatteredRay {
            ray: Ray::new(hit.p, target),
            attenuation: color_to_vector3(self.albedo),
        })
    }
}

#[derive(Debug, Clone)]
struct Metal {
    albedo: Rgb<u8>,
    roughness: f32,
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let reflected = reflect(ray.direction.normalize(), hit.normal);
        if reflected.dot(&hit.normal) > 0.0 {
            Some(ScatteredRay {
                ray: Ray::new(hit.p, reflected + self.roughness * random_in_unit_square()),
                attenuation: color_to_vector3(self.albedo),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct Dialectric {
    ior: f32,
}

impl Material for Dialectric {
    fn scatter(&self, ray: &Ray, hit: &HitResult) -> Option<ScatteredRay> {
        let attenuation = Vector3::new(1.0, 1.0, 1.0);

        let reflected = reflect(ray.direction, hit.normal);
        let dot = ray.direction.dot(&hit.normal);

        let (outward_normal, ni_over_nt, cosine) = if dot > 0.0 {
            (
                -hit.normal,
                self.ior,
                self.ior * dot / ray.direction.magnitude(),
            )
        } else {
            (-hit.normal, self.ior, -dot / ray.direction.magnitude())
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

fn color(ray: &Ray, scene: &dyn Hit, maxdepth: u32, depth: u32) -> Vector3<f32> {
    if let Some(hit) = scene.hit(&ray, 0.001, std::f32::MAX) {
        if depth >= maxdepth {
            return Vector3::new(0.0, 0.0, 0.0);
        }

        if let Some(scattered) = hit.material.scatter(&ray, &hit) {
            return scattered.attenuation.component_mul(&color(
                &scattered.ray,
                scene,
                maxdepth,
                depth + 1,
            ));
        }
    }

    let unit_direction = ray.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);

    (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0)
}

fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let matches = App::new("rtxon")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Simple raytracer built as a learning exercise in Rust")
        .author("Maxwell Koo <mjkoo90@gmail.com>")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Image file to output to")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .value_name("WIDTH")
                .help("Width of the image to output")
                .takes_value(true)
                .default_value("200"),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .value_name("HEIGHT")
                .help("Height of the image to output")
                .takes_value(true)
                .default_value("100"),
        )
        .arg(
            Arg::with_name("samples")
                .short("s")
                .long("samples")
                .value_name("SAMPLES")
                .help("Number of samples per output pixel")
                .takes_value(true)
                .default_value("100"),
        )
        .arg(
            Arg::with_name("maxdepth")
                .short("d")
                .long("maxdepth")
                .value_name("DEPTH")
                .help("Maximum recursion depth")
                .takes_value(true)
                .default_value("50"),
        )
        .get_matches();

    let output = matches
        .value_of("output")
        .expect("Output filename required");
    let width = value_t_or_exit!(matches.value_of("width"), u32);
    let height = value_t_or_exit!(matches.value_of("height"), u32);
    let samples = value_t_or_exit!(matches.value_of("samples"), u32);
    let maxdepth = value_t_or_exit!(matches.value_of("maxdepth"), u32);

    info!(
        "Rendering to {} ({}x{}), {} samples, {} depth",
        &output, width, height, samples, maxdepth
    );

    let scene: Vec<Box<dyn Hit>> = vec![
        Box::new(Sphere {
            center: Point3::new(0.0, 0.0, -1.0),
            radius: 0.5,
            material: Rc::new(Lambertian {
                albedo: Rgb([0xcc, 0x4c, 0x4c]),
            }),
        }),
        Box::new(Sphere {
            center: Point3::new(0.0, -100.5, -1.0),
            radius: 100.0,
            material: Rc::new(Lambertian {
                albedo: Rgb([0xcc, 0xcc, 0x00]),
            }),
        }),
        Box::new(Sphere {
            center: Point3::new(1.0, 0.0, -1.0),
            radius: 0.5,
            material: Rc::new(Metal {
                albedo: Rgb([0xcc, 0x99, 0x33]),
                roughness: 0.3,
            }),
        }),
        Box::new(Sphere {
            center: Point3::new(-1.0, 0.0, -1.0),
            radius: 0.5,
            material: Rc::new(Dialectric { ior: 1.5 }),
        }),
        Box::new(Sphere {
            center: Point3::new(-1.0, 0.0, -1.0),
            radius: -0.45,
            material: Rc::new(Dialectric { ior: 1.5 }),
        }),
    ];

    let camera = Camera {
        origin: Point3::new(0.0, 0.0, 0.0),
        lower_left_corner: Point3::new(-2.0, -1.0, -1.0),
        horizontal: Vector3::new(4.0, 0.0, 0.0),
        vertical: Vector3::new(0.0, 2.0, 0.0),
    };

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let mut c = Vector3::new(0.0, 0.0, 0.0);

        for _ in 0..samples {
            let u = (x as f32 + random::<f32>()) / width as f32;
            let v = 1.0 - (y as f32 + random::<f32>()) / height as f32;

            let ray = camera.get_ray(u, v);
            c += color(&ray, &scene, maxdepth, 0)
        }

        vector3_to_color(c / (samples as f32))
    });

    img.save(output).map_err(Error::from)
}
