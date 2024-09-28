use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/tile.tif";

fn main() {
    println!("Example: cloudtiff + filesystem");
    
    // File access
    println!("Opening `{SAMPLE_COG}`");
    let file = File::open(SAMPLE_COG).unwrap();
    let reader = &mut BufReader::new(file);

    // CloudTiff indexing
    let t_cog = Instant::now();
    let cog = CloudTiff::open(reader).unwrap();
    println!("Indexed COG in {}us", t_cog.elapsed().as_micros());
    println!("{cog}");

    // Tile extraction
    let t_tile = Instant::now();
    let tile = cog.get_tile_at_lat_lon(reader, 0, 54.55, -127.78).unwrap();
    println!("Got tile in {}us", t_tile.elapsed().as_micros());
    println!("{}", tile);

    // Image output
    let img: DynamicImage = tile.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
