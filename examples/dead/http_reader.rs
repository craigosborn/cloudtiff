use cloudtiff::{CloudTiff, HttpReader};
use image::DynamicImage;
use std::time::Instant;
use tokio;
use tracing_subscriber;

const URL: &str = "https://sentinel-cogs.s3.amazonaws.com/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/http_reader_preview.tif";


#[cfg(not(all(feature = "fs", feature = "async")))]
compile_error!("This example requires the ['fs','async'] features to be enabled.");

#[tokio::main]
async fn main() {
    println!("Example: cloudtiff + Async HTTP");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Set the maximum log level
        .with_thread_ids(true)
        .init();

    // Thread-safe Async HTTP Byte Range Reader
    let reader = HttpReader::new(URL).unwrap();

    handler(reader.clone()).await;
}

async fn handler(reader: HttpReader) -> CloudTiff {
    // COG
    let t_cog = Instant::now();
    let cog = CloudTiff::open_par(reader.clone()).await.unwrap();
    println!("Indexed COG in {}ms", t_cog.elapsed().as_millis());
    println!("{cog}");

    // Preview
    let t_preview = Instant::now();
    let preview = cog
        .render_image_with_mp_limit_par(reader.clone(), 1.0)
        .await
        .unwrap();
    println!(
        "Got preview in {:.6} seconds",
        t_preview.elapsed().as_secs_f64()
    );
    println!("{}", preview);

    // // Metrics
    // let ranges = reader.lock().await.requested_ranges().await;
    // let requests = ranges.len();
    // let byte_count: u64 = ranges.iter().map(|r| r.end - r.start).sum();
    // println!("Made {requests} requests and downloaded {byte_count} bytes.");

    // Image
    let img: DynamicImage = preview.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
    cog.clone()
}
