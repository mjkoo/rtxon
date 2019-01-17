use std::rc::Rc;
use std::time::Instant;

use bvh::ray::Ray;
use clap::{value_t_or_exit, App, Arg};
use failure::Error;
use image::{ImageBuffer, Rgb};
use log::info;
use nalgebra::{Point3, Vector3};
use pbr::ProgressBar;
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

fn generate_scene() -> Scene {
    let mut scene: Scene = vec![];
    scene.push(Box::new(Sphere {
        center: Point3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: Rc::new(Lambertian {
            albedo: Rgb([0x7f, 0x7f, 0x7f]),
        }),
    }));

    let avoid = Vector3::new(4.0, 0.2, 0.0);
    for a in -11..11 {
        for b in -11..11 {
            let center = Point3::new(
                (a as f32) + 0.9 * random::<f32>(),
                0.2,
                (b as f32) + 0.9 * random::<f32>(),
            );
            let choose_mat = random::<f32>();

            if (center - avoid).coords.magnitude() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = vector3_to_color(Vector3::new(
                        random::<f32>() * random::<f32>(),
                        random::<f32>() * random::<f32>(),
                        random::<f32>() * random::<f32>(),
                    ));

                    scene.push(Box::new(Sphere {
                        center,
                        radius: 0.2,
                        material: Rc::new(Lambertian { albedo }),
                    }));
                } else if choose_mat < 0.95 {
                    let albedo = vector3_to_color(Vector3::new(
                        0.5 * (1.0 + random::<f32>()),
                        0.5 * (1.0 + random::<f32>()),
                        0.5 * (1.0 + random::<f32>()),
                    ));
                    let roughness = 0.5 * random::<f32>();

                    scene.push(Box::new(Sphere {
                        center,
                        radius: 0.2,
                        material: Rc::new(Metal { albedo, roughness }),
                    }));
                } else {
                    let albedo = Rgb([0xff, 0xff, 0xff]);
                    let ior = 1.5;

                    scene.push(Box::new(Sphere {
                        center,
                        radius: 0.2,
                        material: Rc::new(Dialectric { albedo, ior }),
                    }));
                }
            }
        }
    }

    scene.push(Box::new(Sphere {
        center: Point3::new(0.0, 1.0, 0.0),
        radius: 1.0,
        material: Rc::new(Dialectric {
            albedo: Rgb([0xff, 0xff, 0xff]),
            ior: 1.5,
        }),
    }));

    scene.push(Box::new(Sphere {
        center: Point3::new(-4.0, 1.0, 0.0),
        radius: 1.0,
        material: Rc::new(Lambertian {
            albedo: Rgb([0x66, 0x33, 0x19]),
        }),
    }));

    scene.push(Box::new(Sphere {
        center: Point3::new(4.0, 1.0, 0.0),
        radius: 1.0,
        material: Rc::new(Metal {
            albedo: Rgb([0xb3, 0x99, 0x7f]),
            roughness: 0.0,
        }),
    }));

    scene
}

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .write_style(env_logger::WriteStyle::Auto)
        .init();

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

    let scene = generate_scene();

    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let aspect_ratio = (width as f32) / (height as f32);
    let focal_length = (lookfrom - lookat).magnitude();

    let camera = Camera::new(
        lookfrom,
        lookat,
        Vector3::y(),
        20.0,
        aspect_ratio,
        0.1,
        focal_length,
    );

    let mut pb = ProgressBar::new((width * height) as u64);
    let start = Instant::now();

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let mut c = Vector3::new(0.0, 0.0, 0.0);

        for _ in 0..samples {
            let u = (x as f32 + random::<f32>()) / width as f32;
            let v = 1.0 - (y as f32 + random::<f32>()) / height as f32;

            let ray = camera.get_ray(u, v);
            c += color(&ray, &scene, maxdepth, 0)
        }

        pb.inc();
        vector3_to_color(c / (samples as f32))
    });

    let end = Instant::now();
    pb.finish();

    info!("Rendered in {:?}", end.duration_since(start));

    img.save(output).map_err(Error::from)
}
