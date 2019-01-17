use std::rc::Rc;

use bvh::ray::Ray;
use clap::{value_t_or_exit, App, Arg};
use failure::Error;
use image::{ImageBuffer, Rgb};
use log::info;
use nalgebra::{Point3, Vector3};
use rand::random;

mod camera;
mod materials;
mod shapes;
mod utils;

use crate::camera::Camera;
use crate::materials::{Dialectric, Lambertian, Metal};
use crate::shapes::{Scene, Shape, Sphere};
use crate::utils::vector3_to_color;

fn color(ray: &Ray, scene: &Scene, maxdepth: u32, depth: u32) -> Vector3<f32> {
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

    let scene: Scene = vec![
        Box::new(Sphere {
            center: Point3::new(0.0, 0.0, -1.0),
            radius: 0.5,
            material: Rc::new(Lambertian {
                albedo: Rgb([0x19, 0x33, 0x7f]),
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
            material: Rc::new(Dialectric {
                albedo: Rgb([255, 255, 255]),
                ior: 1.5,
            }),
        }),
        Box::new(Sphere {
            center: Point3::new(-1.0, 0.0, -1.0),
            radius: -0.45,
            material: Rc::new(Dialectric {
                albedo: Rgb([255, 255, 255]),
                ior: 1.5,
            }),
        }),
    ];

    let origin = Point3::new(3.0, 3.0, 2.0);
    let lookat = Point3::new(0.0, 0.0, -1.0);
    let aspect_ratio = (width as f32) / (height as f32);
    let focal_length = (origin - lookat).magnitude();

    let camera = Camera::new(
        origin,
        lookat,
        Vector3::y(),
        20.0,
        aspect_ratio,
        2.0,
        focal_length,
    );

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
