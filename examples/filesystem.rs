use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/tile.tif";

fn main() {
    // File access
    println!("Opening {SAMPLE_COG}");
    let file = File::open(SAMPLE_COG).unwrap();
    let reader = &mut BufReader::new(file);

    // CloudTiff parsing
    let t0 = Instant::now();
    let cog = CloudTiff::open(reader).unwrap();
    println!("Decoded in {:.6}s", t0.elapsed().as_secs_f64());
    println!("{cog}");

    // Tile indexing
    let tile = cog.get_tile(cog.max_level(), 0, 0).unwrap();
    println!("{}", tile);

    // Raster extraction
    let raster = tile.extract(reader).unwrap();
    println!("{}", raster);

    // Image output
    let img: DynamicImage = raster.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
