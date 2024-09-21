use cloudtiff::CloudTiff;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";

fn main() {
    let file = File::open(SAMPLE_COG).unwrap();
    let reader = &mut BufReader::new(file);
    println!("Opening {SAMPLE_COG}");
    
    let t0 = Instant::now();
    let cog = CloudTiff::open(reader).unwrap();
    println!("Opened in {:.6}s", t0.elapsed().as_secs_f64());

    println!("{cog}");
}
