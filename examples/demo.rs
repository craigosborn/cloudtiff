use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/demo.jpg";
const PREVIEW_MEGAPIXELS: f64 = 10.0;

fn main() {
    println!("Example: demo");

    let file = File::open(SAMPLE_COG).unwrap();
    save_preview(file);
}

fn save_preview(file: File) {
    let t_cog = Instant::now();
    let mut reader = BufReader::new(file); // TODO shouldnt need this
    let cog = CloudTiff::open(&mut reader).unwrap();
    println!(
        "Opened COG in {:.3}ms",
        t_cog.elapsed().as_micros() as f64 / 1000.0
    );

    let t_preview = Instant::now();
    let preview = cog
        .renderer()
        .with_mp_limit(PREVIEW_MEGAPIXELS)
        .with_reader(reader)
        .render()
        .unwrap();
    println!(
        "Got preview in {:.3}ms",
        t_preview.elapsed().as_micros() as f64 / 1000.0
    );

    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
