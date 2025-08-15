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
const CROP: [f64; 4] = [0.0,0.0,1.0,1.0]; // [min_x, min_y, max_x, max_y] in relative (0-1) units
const OUTPUT_FILE: &str = "data/crop.png";
const MAX_MEGAPIXELS: f64 = 1.0;

#[tokio::main]
async fn main() {
    println!("Example: cloudtiff async crop");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
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
    let crop = cog
        .renderer()
        .of_crop(CROP[0], CROP[1], CROP[2], CROP[3])
        .with_mp_limit(MAX_MEGAPIXELS)
        .with_async_reader(thread_safe_file)
        .render_async()
        .await
        .unwrap();
    println!(
        "Got crop in {:.3}ms",
        t0.elapsed().as_micros() as f64 / 1000.0
    );

    let img: DynamicImage = crop.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}
