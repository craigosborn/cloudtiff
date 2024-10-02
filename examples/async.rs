#[cfg(not(feature = "async"))]
compile_error!("This example requires the ['image', 'async'] features");

use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::sync::Arc;
use std::time::Instant;
use tokio;
use tokio::fs::File;
use tokio::sync::Mutex;
use tracing_subscriber;

const SAMPLE_COG: &str = "data/sample.tif";
const OUTPUT_FILE: &str = "data/async.jpg";
const PREVIEW_MEGAPIXELS: f64 = 1.0;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    println!("Example: async");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    let t_cog = Instant::now();
    let mut file = File::open(SAMPLE_COG).await.unwrap();
    let cog = CloudTiff::open_async(&mut file).await.unwrap();
    println!(
        "Opened COG in {:.3}ms",
        t_cog.elapsed().as_micros() as f64 / 1000.0
    );

    let t0 = Instant::now();
    let thread_safe_file = Arc::new(Mutex::new(file));
    let preview = cog
        .renderer()
        .with_mp_limit(PREVIEW_MEGAPIXELS)
        .with_async_reader(thread_safe_file)
        // .with_image_region((0.0,0.0,0.1,0.1))
        .render_async()
        .await
        .unwrap();
    println!(
        "Got preview in {:.3}ms",
        t0.elapsed().as_micros() as f64 / 1000.0
    );

    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
