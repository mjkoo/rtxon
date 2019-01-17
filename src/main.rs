use bvh::ray::Ray;
use clap::{value_t_or_exit, App, Arg};
use failure::Error;
use image::{ImageBuffer, Rgb};
use log::info;
use nalgebra::{Point3, Unit, Vector3};

fn f32_to_u8(f: f32) -> u8 {
    (f * 255.99) as u8
}

fn hit_sphere(center: Point3<f32>, radius: f32, ray: &Ray) -> f32 {
    let oc = ray.origin - center;
    let a = ray.direction.dot(&ray.direction);
    let b = 2.0 * oc.dot(&ray.direction);
    let c = oc.dot(&oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        -1.0
    } else {
        (-b - discriminant.sqrt()) / (2.0 * a)
    }
}

fn point_at_parameter(ray: &Ray, t: f32) -> Point3<f32> {
    ray.origin + t * Unit::new_normalize(ray.direction).into_inner()
}

fn vector3_to_color(v: Vector3<f32>) -> Rgb<u8> {
    Rgb([f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z)])
}

fn color(ray: &Ray) -> Rgb<u8> {
    let center = Point3::new(0.0, 0.0, -1.0);
    let t = hit_sphere(center, 0.5, &ray);
    if t > 0.0 {
        let n = Unit::new_normalize(point_at_parameter(&ray, t) - center).into_inner();
        let color = 0.5 * Vector3::new(n.x + 1.0, n.y + 1.0, n.z + 1.0);
        return vector3_to_color(color);
    }

    let unit_direction = Unit::new_normalize(ray.direction).into_inner();
    let t = 0.5 * (unit_direction.y + 1.0);
    let color = (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);
    vector3_to_color(color)
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
        .get_matches();

    let output = matches
        .value_of("output")
        .expect("Output filename required");
    let width = value_t_or_exit!(matches.value_of("width"), u32);
    let height = value_t_or_exit!(matches.value_of("height"), u32);

    info!("Rendering to {} ({}x{})", &output, width, height);

    let origin = Point3::new(0.0, 0.0, 0.0);
    let lower_left_corner = Vector3::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let u = x as f32 / width as f32;
        let v = y as f32 / height as f32;

        let ray = Ray::new(origin, lower_left_corner + u * horizontal + (1.0 - v) * vertical);
        color(&ray)
    });

    img.save(output).map_err(Error::from)
}
