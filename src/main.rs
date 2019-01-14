use clap::{value_t_or_exit, App, Arg};
use image::{ImageBuffer, Rgb};

fn main() {
    let matches = App::new("rtxon")
        .version("0.1.0")
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

    println!("{}, {}x{}", &output, width, height);

    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let r = (x as f64) / (width as f64);
        let g = (y as f64) / (height as f64);
        let b = 0.2;

        Rgb([(r * 255.99) as u8, (g * 255.99) as u8, (b * 255.99) as u8])
    });

    img.save(output).expect("Could not write image");
}
