use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/demo.jpg";

fn main() {
    let file = File::open(SAMPLE_COG).unwrap();
    save_preview(file);
}

fn save_preview(file: File) {
    let reader = &mut BufReader::new(&file);
    let cog = CloudTiff::open(reader).unwrap();

    let preview = cog
        .renderer()
        .with_mp_limit(1.0)
        .with_reader(file)
        .render()
        .unwrap();

    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
}