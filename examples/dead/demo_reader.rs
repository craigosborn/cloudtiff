use cloudtiff::CloudTiff;
use cloudtiff::PathReader;
use image::DynamicImage;
use std::time::Instant;
use tokio;
use tracing_subscriber;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/preview_filereader.jpg";

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    let mut reader: PathReader = PathReader::new(SAMPLE_COG);
    let cog = CloudTiff::open_par(&mut reader).await.unwrap();

    let t0 = Instant::now();
    let preview = cog.render_image_with_mp_limit_par(&mut reader, 10.0).await.unwrap();
    println!("Got preview in {:.3}ms", t0.elapsed().as_micros() as f64 / 1000.0);

    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
}