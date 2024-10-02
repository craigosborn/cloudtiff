use cloudtiff::cog;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

const SAMPLE_COG: &str = "data/sample.tif";

fn main() {
    println!("Example: cloudtiff disect");
    
    // File access
    println!("Opening `{SAMPLE_COG}`");
    let file = File::open(SAMPLE_COG).unwrap();
    let reader = &mut BufReader::new(file);

    println!("Diesecting COG:");
    cog::disect(reader).unwrap();
    reader.seek(SeekFrom::Start(0)).unwrap();
}
