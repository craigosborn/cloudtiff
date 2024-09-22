use cloudtiff::CloudTiff;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";

fn main() {
    // File
    println!("Opening {SAMPLE_COG}");
    let file = File::open(SAMPLE_COG).unwrap();
    let reader = &mut BufReader::new(file);

    // CloudTiff
    let t0 = Instant::now();
    let cog = CloudTiff::open(reader).unwrap();
    println!("Decoded in {:.6}s", t0.elapsed().as_secs_f64());
    println!("{cog}");

    // Tile
    let tile = cog.get_tile(reader, 0, 0, 0).unwrap();
    println!("{}", tile);
}
