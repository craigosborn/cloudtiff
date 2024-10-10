#[cfg(not(feature = "image"))]
compile_error!("This example requires the 'image' feature");

use cloudtiff::{Encoder, Region};
use image;
use std::fs::File;

const INPUT_FILE: &str = "data/demo.jpg";
const OUTPUT_COG: &str = "data/encode.tif";

fn main() {
    println!("Example: cloudtiff encode");

    let img = image::open(INPUT_FILE).unwrap();

    let tiepoint = (499980.0, 6100020.0, 0.0);
    let pixel_scale = (10.0, 10.0, 10.0);
    let full_dim = (10980, 10980);
    let encoder = Encoder::from_image(&img)
        .unwrap()
        .with_projection(
            32609,
            Region::new(
                tiepoint.0,
                tiepoint.1 - pixel_scale.1 * full_dim.1 as f64,
                tiepoint.0 + pixel_scale.0 * full_dim.0 as f64,
                tiepoint.1,
            ),
        )
        .with_tile_size(256)
        .with_big_tiff(false);

    let mut file = File::create(OUTPUT_COG).unwrap();
    encoder.encode(&mut file).unwrap();
    println!("Saved COG to {OUTPUT_COG}");
}
