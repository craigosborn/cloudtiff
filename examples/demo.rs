#[cfg(feature = "image")]
use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::time::Instant;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/demo.jpg";
const PREVIEW_MEGAPIXELS: f64 = 1.0;

fn main() {
    println!("Example: cloudtiff demo");

    let file = File::open(SAMPLE_COG).unwrap();
    save_preview(file);
}

fn save_preview(mut file: File) {
    let t_cog = Instant::now();
    let cog = CloudTiff::open(&mut file).unwrap();
    println!(
        "Opened COG in {:.3}ms",
        t_cog.elapsed().as_micros() as f64 / 1000.0
    );

    // optional on unix
    let file = std::sync::Mutex::new(file);

    let t_preview = Instant::now();
    let preview = cog
        .renderer()
        .with_mp_limit(PREVIEW_MEGAPIXELS)
        .with_reader(&file)
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
