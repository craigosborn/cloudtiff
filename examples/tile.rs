#![cfg(feature = "image")]

use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::env;
use std::fs::File;
use std::sync::Mutex;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const LEVEL: usize = 0;
const ROW: usize = 0;
const COL: usize = 0;
const OUTPUT_FILE: &str = "data/tile.png";

// Use
// cargo run --example tile -- path/to/some/cog.tif level row column output/tile.tif

fn main() {
    println!("Example: cloudtiff tile");

    let args: Vec<String> = env::args().collect();
    let input_cog = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from(SAMPLE_COG)
    };
    let (z, y, x) = if args.len() > 4 {
        (
            args[2].parse().unwrap(),
            args[3].parse().unwrap(),
            args[4].parse().unwrap(),
        )
    } else {
        (LEVEL, ROW, COL)
    };
    let output_file = if args.len() > 5 {
        args[5].clone()
    } else {
        String::from(OUTPUT_FILE)
    };

    let mut file = File::open(input_cog).unwrap();
    let cog = CloudTiff::open(&mut file).unwrap();

    // optional on unix
    let file = Mutex::new(file);

    let t_tile = Instant::now();
    let tile = cog
        .renderer()
        .of_tile(x, y, z)
        .with_reader(&file)
        .render()
        .unwrap();
    println!(
        "Got tile in {:.3}ms",
        t_tile.elapsed().as_micros() as f64 / 1000.0
    );

    let img: DynamicImage = tile.try_into().unwrap();
    img.save(output_file).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
