use cloudtiff::CloudTiff;
use image::DynamicImage;
use tokio::fs::File;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;
use tokio;
use tracing_subscriber;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/preview_async.jpg";

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    let file = Arc::new(Mutex::new(File::open(SAMPLE_COG).await.unwrap()));
    let cog = CloudTiff::open_async(file.clone()).await.unwrap();

    let t0 = Instant::now();
    let preview = cog.render_image_with_mp_limit_async(file.clone(), 10.0).await.unwrap();
    println!("Got preview in {:.3}ms", t0.elapsed().as_micros() as f64 / 1000.0);

    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
}