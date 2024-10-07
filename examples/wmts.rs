#[cfg(not(feature = "image"))]
compile_error!("This example requires the 'image' feature");

use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/wmts.jpg";
const TILE_SIZE: u32 = 512;

fn main() {
    println!("Example: cloudtiff wmts");

    // COG
    println!("Opening `{SAMPLE_COG}`");
    let mut file = File::open(SAMPLE_COG).unwrap();
    let cog = CloudTiff::open(&mut file).unwrap();

    // Tile
    let t_tile = Instant::now();
    let tile = cog
        .renderer()
        .with_exact_resolution((TILE_SIZE, TILE_SIZE))
        .of_wmts_tile(1188, 2608, 13)
        .unwrap()
        .with_reader(file)
        .render()
        .unwrap();
    println!(
        "Rendered tile in {:.3}ms",
        t_tile.elapsed().as_secs_f64() * 1000.0
    );
    println!("Tile: {tile}");

    // Output
    let img: DynamicImage = tile.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
