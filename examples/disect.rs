use cloudtiff;
use std::env;
use std::fs::File;
use std::io::BufReader;

const SAMPLE_COG: &str = "data/sample.tif";

fn main() {
    println!("Example: cloudtiff disect");

    let args: Vec<String> = env::args().chain(vec![SAMPLE_COG.to_string()]).collect();
    let path = &args[1];

    // File access
    println!("Opening `{path}`");
    let file = File::open(path).unwrap();
    let reader = &mut BufReader::new(file);

    println!("Diesecting COG:");
    cloudtiff::disect(reader).unwrap();

}
