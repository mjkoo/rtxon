use std::sync::{Arc, Mutex};
use std::time::Instant;

use clap::{value_t_or_exit, App, Arg};
use failure::Error;
use log::info;
use pbr::ProgressBar;
use rand::random;

mod camera;
mod image;
mod materials;
mod shapes;
mod types;

use crate::camera::Camera;
use crate::materials::{Dialectric, Lambertian, Metal};
use crate::shapes::{Scene, Shape, Sphere};
use crate::types::{Color, Point3, Ray, Scalar, Vector3};

fn color(ray: &Ray, scene: &Scene, maxdepth: u32, depth: u32) -> Color {
    if let Some(hit) = scene.hit(&ray, 0.001, std::f32::MAX) {
        if depth >= maxdepth {
            return Color::new(0.0, 0.0, 0.0, 1.0);
        }

        if let Some(scattered) = hit.material.scatter(&ray, &hit) {
            return scattered.attenuation * color(&scattered.ray, scene, maxdepth, depth + 1);
        }
    }

    let unit_direction = ray.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    let c = (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);

    c.into()
}

fn generate_scene() -> Scene {
    let mut scene: Scene = vec![];
    scene.push(Arc::new(Sphere {
        center: Point3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: Arc::new(Lambertian {
            albedo: Color::new(0.5, 0.5, 0.5, 1.0),
        }),
    }));

    let avoid = Vector3::new(4.0, 0.2, 0.0);
    for a in -11..11 {
        for b in -11..11 {
            let center = Point3::new(
                (a as Scalar) + 0.9 * random::<Scalar>(),
                0.2,
                (b as Scalar) + 0.9 * random::<Scalar>(),
            );
            let choose_mat = random::<Scalar>();

            if (center - avoid).coords.magnitude() > 0.9 {
                if choose_mat < 0.8 {
                    let albedo = Color::new(
                        random::<Scalar>() * random::<Scalar>(),
                        random::<Scalar>() * random::<Scalar>(),
                        random::<Scalar>() * random::<Scalar>(),
                        1.0,
                    );

                    scene.push(Arc::new(Sphere {
                        center,
                        radius: 0.2,
                        material: Arc::new(Lambertian { albedo }),
                    }));
                } else if choose_mat < 0.95 {
                    let albedo = Color::new(
                        0.5 * (1.0 + random::<Scalar>()),
                        0.5 * (1.0 + random::<Scalar>()),
                        0.5 * (1.0 + random::<Scalar>()),
                        1.0,
                    );
                    let roughness = 0.5 * random::<Scalar>();

                    scene.push(Arc::new(Sphere {
                        center,
                        radius: 0.2,
                        material: Arc::new(Metal { albedo, roughness }),
                    }));
                } else {
                    let albedo = Color::new(1.0, 1.0, 1.0, 1.0);
                    let ior = 1.5;

                    scene.push(Arc::new(Sphere {
                        center,
                        radius: 0.2,
                        material: Arc::new(Dialectric { albedo, ior }),
                    }));
                }
            }
        }
    }

    scene.push(Arc::new(Sphere {
        center: Point3::new(0.0, 1.0, 0.0),
        radius: 1.0,
        material: Arc::new(Dialectric {
            albedo: Color::new(1.0, 1.0, 1.0, 1.0),
            ior: 1.5,
        }),
    }));

    scene.push(Arc::new(Sphere {
        center: Point3::new(-4.0, 1.0, 0.0),
        radius: 1.0,
        material: Arc::new(Lambertian {
            albedo: Color::new(0.4, 0.2, 0.1, 1.0),
        }),
    }));

    scene.push(Arc::new(Sphere {
        center: Point3::new(4.0, 1.0, 0.0),
        radius: 1.0,
        material: Arc::new(Metal {
            albedo: Color::new(0.7, 0.6, 0.5, 1.0),
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
    let aspect_ratio = (width as Scalar) / (height as Scalar);
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

    let pb = Arc::new(Mutex::new(ProgressBar::new(u64::from(width * height))));
    let start = Instant::now();

    let img = {
        let pb = pb.clone();

        image::Image::from_fn(
            width as usize,
            height as usize,
            move |x, y| -> ::image::Rgba<u8> {
                let mut c = Color::new(0.0, 0.0, 0.0, 0.0);

                for _ in 0..samples {
                    let u = (x as Scalar + random::<Scalar>()) / width as Scalar;
                    let v = 1.0 - (y as Scalar + random::<Scalar>()) / height as Scalar;

                    let ray = camera.get_ray(u, v);
                    c += color(&ray, &scene, maxdepth, 0)
                }

                pb.lock().unwrap().inc();
                c /= samples as Scalar;

                c.into()
            },
        )
    };

    img.save(output).map_err(Error::from)?;

    let end = Instant::now();
    pb.lock().unwrap().finish();

    info!("Finished in {:?}", end.duration_since(start));

    Ok(())
}
