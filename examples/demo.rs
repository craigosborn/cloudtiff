use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;

const SAMPLE_COG: &str = "data/sample.tif";

fn main() {
    let file = File::open(SAMPLE_COG).unwrap();
    save_preview(file);
}

fn save_preview(file: File) {
    let reader = &mut BufReader::new(file);
    let cog = CloudTiff::open(reader).unwrap();

    let preview = cog.render_image_with_mp_limit(reader, 1.0).unwrap();

    let img: DynamicImage = preview.try_into().unwrap();
    img.save("data/preview.jpg").unwrap();
}