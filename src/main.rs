use std::cmp::Ordering;

use bvh::ray::Ray;
use clap::{value_t_or_exit, App, Arg};
use failure::Error;
use image::{ImageBuffer, Rgb};
use log::info;
use nalgebra::{Point3, Unit, Vector3};

fn f32_to_u8(f: f32) -> u8 {
    (f * 255.99) as u8
}

fn vector3_to_color(v: Vector3<f32>) -> Rgb<u8> {
    Rgb([f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z)])
}

fn point_at_parameter(ray: &Ray, t: f32) -> Point3<f32> {
    ray.origin + t * Unit::new_normalize(ray.direction).into_inner()
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
        Ray::new(self.origin, self.lower_left_corner.coords + u * self.horizontal + v * self.vertical)
    }
}

#[derive(Debug, Clone)]
struct HitResult {
    t: f32,
    p: Point3<f32>,
    normal: Vector3<f32>,
}

trait Hit {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitResult>;
}

#[derive(Debug, Clone)]
struct Sphere {
    center: Point3<f32>,
    radius: f32,
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

        if discriminant > 0.0 && t1 > t_min && t1 < t_max {
            let p = point_at_parameter(&ray, t1);
            let normal = (p - self.center) / self.radius;
            Some(HitResult { t: t1, p, normal })
        } else if discriminant > 0.0 && t2 > t_min && t2 < t_max {
            let p = point_at_parameter(&ray, t2);
            let normal = (p - self.center) / self.radius;
            Some(HitResult { t: t2, p, normal })
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

fn color(ray: &Ray, scene: &dyn Hit) -> Vector3<f32> {
    if let Some(hit) = scene.hit(&ray, 0.0, std::f32::MAX) {
        return 0.5 * Vector3::new(hit.normal.x + 1.0, hit.normal.y + 1.0, hit.normal.z + 1.0);
    }

    let unit_direction = Unit::new_normalize(ray.direction).into_inner();
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
        .get_matches();

    let output = matches
        .value_of("output")
        .expect("Output filename required");
    let width = value_t_or_exit!(matches.value_of("width"), u32);
    let height = value_t_or_exit!(matches.value_of("height"), u32);
    let samples = value_t_or_exit!(matches.value_of("samples"), u32);

    info!("Rendering to {} ({}x{})", &output, width, height);

    let scene: Vec<Box<dyn Hit>> = vec![
        Box::new(Sphere {
            center: Point3::new(0.0, 0.0, -1.0),
            radius: 0.5,
        }),
        Box::new(Sphere {
            center: Point3::new(0.0, -100.5, -1.0),
            radius: 100.0,
        }),
    ];

    let camera = Camera{
        origin: Point3::new(0.0, 0.0, 0.0),
        lower_left_corner: Point3::new(-2.0, -1.0, -1.0),
        horizontal: Vector3::new(4.0, 0.0, 0.0),
        vertical: Vector3::new(0.0, 2.0, 0.0),
    };

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let mut c = Vector3::new(0.0, 0.0, 0.0);

        for _ in 0..samples {
            let u = (x as f32 + rand::random::<f32>()) / width as f32;
            let v = 1.0 - (y as f32 + rand::random::<f32>()) / height as f32;

            let ray = camera.get_ray(u, v);
            c += color(&ray, &scene)
        }

        vector3_to_color(c / (samples as f32))
    });

    img.save(output).map_err(Error::from)
}
