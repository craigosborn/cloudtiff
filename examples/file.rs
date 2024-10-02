#[cfg(not(feature = "image"))]
compile_error!("This example requires the 'image' feature");

use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/filesystem.jpg";
const PREVIEW_MEGAPIXELS: f64 = 1.0;

fn main() {
    println!("Example: cloudtiff file");

    // File access
    println!("Opening `{SAMPLE_COG}`");
    let file = File::open(SAMPLE_COG).unwrap();
    let mut reader = BufReader::new(file);

    // CloudTiff indexing
    let t_cog = Instant::now();
    let cog = CloudTiff::open(&mut reader).unwrap();
    println!("Indexed COG in {}us", t_cog.elapsed().as_micros());
    println!("{cog}");

    // Tile extraction
    let t_tile = Instant::now();
    let preview = cog
        .renderer()
        .with_mp_limit(PREVIEW_MEGAPIXELS)
        .with_reader(reader)
        .render()
        .unwrap();
    println!("Got preview in {:.3}ms", t_tile.elapsed().as_secs_f32() * 1e3);
    println!("{}", preview);

    // Image output
    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
